import { useState, useCallback } from "react";
import {
  fetchEscrow, fetchTimeLeft,
  txDeposit, txApprove, txRefund,
  EscrowRecord,
} from "../lib/opnet";
import { NATIVE_BTC } from "../lib/constants";

export function useEscrow() {
  const [escrow,  setEscrow]  = useState<EscrowRecord | null>(null);
  const [timeLeft, setTimeLeft] = useState<number>(0);
  const [loading, setLoading] = useState(false);
  const [error,   setError]   = useState<string | null>(null);

  const load = useCallback(async (id: bigint) => {
    setLoading(true);
    setError(null);
    try {
      const [rec, tl] = await Promise.all([fetchEscrow(id), fetchTimeLeft(id)]);
      setEscrow(rec);
      setTimeLeft(tl);
      if (!rec) setError("Escrow not found");
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to load");
    } finally {
      setLoading(false);
    }
  }, []);

  const deposit = useCallback(async (
    seller: string, token: string, amount: bigint
  ) => {
    setLoading(true);
    setError(null);
    try {
      const isNative = token === NATIVE_BTC || token === "";
      const tok = isNative ? NATIVE_BTC : token;
      const result = await txDeposit(seller, tok, amount, isNative);
      return result.txId;
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Deposit failed");
    } finally {
      setLoading(false);
    }
  }, []);

  const approve = useCallback(async (id: bigint) => {
    setLoading(true);
    setError(null);
    try {
      const result = await txApprove(id);
      return result.txId;
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Approve failed");
    } finally {
      setLoading(false);
    }
  }, []);

  const refund = useCallback(async (id: bigint) => {
    setLoading(true);
    setError(null);
    try {
      const result = await txRefund(id);
      return result.txId;
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Refund failed");
    } finally {
      setLoading(false);
    }
  }, []);

  return { escrow, timeLeft, loading, error, load, deposit, approve, refund };
}
