use opnet::prelude::*;
use opnet::storage::{StorageMap, StorageValue};
use opnet::types::{Address, U256};

/// VibeEscrow — trustless P2P escrow on OP_NET Bitcoin L1
/// Status: 0 = Pending, 1 = Released, 2 = Refunded

#[derive(Encode, Decode, Clone)]
pub struct EscrowRecord {
    pub buyer:           Address,
    pub seller:          Address,
    pub token:           Address,  // Address::zero() = native BTC
    pub amount:          U256,
    pub deadline:        u64,      // Unix timestamp (deposit_time + 7 days)
    pub status:          u8,       // 0=Pending 1=Released 2=Refunded
    pub buyer_approved:  bool,
    pub seller_approved: bool,
}

#[opnet_contract]
pub struct VibeEscrow {
    escrows: StorageMap<U256, EscrowRecord>,
    next_id: StorageValue<U256>,
    owner:   StorageValue<Address>,
}

#[opnet_events]
pub enum EscrowEvent {
    EscrowCreated {
        id: U256, buyer: Address, seller: Address,
        amount: U256, token: Address, deadline: u64,
    },
    Approved { id: U256, approver: Address },
    Released { id: U256 },
    Refunded { id: U256 },
}

#[opnet_impl]
impl VibeEscrow {

    pub fn constructor(&mut self) {
        self.owner.set(msg_sender());
        self.next_id.set(U256::zero());
    }

    // ── DEPOSIT ──────────────────────────────────
    /// Create escrow. token == Address::zero() means native BTC.
    /// For BTC: attach value. For OP_20: approve token first.
    #[payable]
    pub fn deposit(
        &mut self,
        seller: Address,
        token:  Address,
        amount: U256,
    ) -> U256 {
        require(seller != Address::zero(), "seller cannot be zero");
        require(seller != msg_sender(),    "buyer == seller");
        require(amount > U256::zero(),     "amount must be > 0");

        let is_native = token == Address::zero();
        if is_native {
            require(msg_value() >= amount, "insufficient BTC attached");
        } else {
            let ok = self.erc20_transfer_from(token, msg_sender(), self_address(), amount);
            require(ok, "token transferFrom failed");
        }

        let id       = self.next_id.get();
        let deadline = block_timestamp() + 7 * 24 * 60 * 60; // +7 days

        let record = EscrowRecord {
            buyer:           msg_sender(),
            seller,
            token,
            amount,
            deadline,
            status:          0,
            buyer_approved:  false,
            seller_approved: false,
        };

        self.escrows.set(id, record.clone());
        self.next_id.set(id + U256::one());

        emit(EscrowEvent::EscrowCreated {
            id,
            buyer:    record.buyer,
            seller:   record.seller,
            amount:   record.amount,
            token:    record.token,
            deadline,
        });

        id
    }

    // ── APPROVE ──────────────────────────────────
    pub fn approve(&mut self, id: U256) {
        let mut record = self.escrows.get(id).expect("escrow not found");
        require(record.status == 0, "escrow not pending");
        require(
            msg_sender() == record.buyer || msg_sender() == record.seller,
            "not a party",
        );

        if msg_sender() == record.buyer {
            record.buyer_approved = true;
        } else {
            record.seller_approved = true;
        }

        emit(EscrowEvent::Approved { id, approver: msg_sender() });

        if record.buyer_approved && record.seller_approved {
            self.do_release(&mut record, id);
        } else {
            self.escrows.set(id, record);
        }
    }

    // ── REFUND ───────────────────────────────────
    pub fn refund(&mut self, id: U256) {
        let mut record = self.escrows.get(id).expect("escrow not found");
        require(record.status == 0,                   "escrow not pending");
        require(block_timestamp() >= record.deadline, "timelock active");
        require(
            msg_sender() == record.buyer
                || msg_sender() == record.seller
                || msg_sender() == self.owner.get(),
            "unauthorized",
        );

        self.do_refund(&mut record, id);
    }

    // ── VIEW ─────────────────────────────────────
    pub fn get_escrow(&self, id: U256) -> Option<EscrowRecord> {
        self.escrows.get(id)
    }

    pub fn get_escrow_count(&self) -> U256 {
        self.next_id.get()
    }

    pub fn get_time_left(&self, id: U256) -> u64 {
        let record = self.escrows.get(id).expect("not found");
        let now = block_timestamp();
        if now >= record.deadline { 0 } else { record.deadline - now }
    }

    // ── INTERNAL ─────────────────────────────────
    fn do_release(&mut self, record: &mut EscrowRecord, id: U256) {
        record.status = 1;
        self.escrows.set(id, record.clone());
        if record.token == Address::zero() {
            transfer_btc(record.seller, record.amount);
        } else {
            self.erc20_transfer(record.token, record.seller, record.amount);
        }
        emit(EscrowEvent::Released { id });
    }

    fn do_refund(&mut self, record: &mut EscrowRecord, id: U256) {
        record.status = 2;
        self.escrows.set(id, record.clone());
        if record.token == Address::zero() {
            transfer_btc(record.buyer, record.amount);
        } else {
            self.erc20_transfer(record.token, record.buyer, record.amount);
        }
        emit(EscrowEvent::Refunded { id });
    }

    fn erc20_transfer_from(&self, token: Address, from: Address, to: Address, amount: U256) -> bool {
        let call_data = encode_call("transferFrom", &[from.into(), to.into(), amount.into()]);
        let result = call_contract(token, call_data, U256::zero());
        decode_bool(result).unwrap_or(false)
    }

    fn erc20_transfer(&self, token: Address, to: Address, amount: U256) -> bool {
        let call_data = encode_call("transfer", &[to.into(), amount.into()]);
        let result = call_contract(token, call_data, U256::zero());
        decode_bool(result).unwrap_or(false)
    }
}