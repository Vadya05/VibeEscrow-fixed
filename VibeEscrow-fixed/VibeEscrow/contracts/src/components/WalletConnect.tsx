import React from "react";
import { WalletState } from "../hooks/useWallet";

interface Props {
  wallet: WalletState & {
    connect: () => Promise<void>;
    refreshBalance: () => Promise<void>;
  };
}

function satsToBTC(sats: bigint): string {
  return (Number(sats) / 1e8).toFixed(8);
}

function shortAddr(addr: string): string {
  return addr.length > 12 ? `${addr.slice(0, 6)}…${addr.slice(-6)}` : addr;
}

export const WalletConnect: React.FC<Props> = ({ wallet }) => {
  if (wallet.connected && wallet.address) {
    return (
      <div className="wallet-connected">
        <span className="wallet-addr">{shortAddr(wallet.address)}</span>
        <span className="wallet-balance">
          ₿ {satsToBTC(wallet.btcBalance)}
        </span>
      </div>
    );
  }

  return (
    <button
      className="btn btn-primary"
      onClick={wallet.connect}
      disabled={wallet.connecting}
    >
      {wallet.connecting ? <span className="spinner" /> : "Connect OP_WALLET"}
    </button>
  );
};
