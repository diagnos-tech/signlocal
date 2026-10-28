/**
 * A React hook around @signlocal/sdk: the setup status (live), and a `sign`
 * that tracks its own progress and error. Copy it into your app; it is ~60
 * lines on purpose, not a package.
 */

// #region site:react
import {
  isSignlocalError,
  onChange,
  status as readStatus,
  type SignlocalError,
  type SignOptions,
  type SignResult,
  type Status,
  sign as signWithSignlocal,
} from "@signlocal/sdk";
import { useCallback, useEffect, useRef, useState } from "react";

export interface SignlocalState {
  /** undefined while the first check runs. */
  readonly status: Status | undefined;
  readonly signing: boolean;
  /** The last failure, except the person cancelling (nothing to show then). */
  readonly error: SignlocalError | undefined;
  /** Resolves with the result, or undefined when it failed (see `error`). */
  readonly sign: (options: SignOptions) => Promise<SignResult | undefined>;
}

export function useSignlocal(): SignlocalState {
  const [status, setStatus] = useState<Status>();
  const [signing, setSigning] = useState(false);
  const [error, setError] = useState<SignlocalError>();
  // Closing the component cancels a signature in progress (the app's window closes too).
  const pending = useRef<AbortController | null>(null);

  useEffect(() => {
    let live = true;
    readStatus().then((s) => live && setStatus(s));
    const stop = onChange(setStatus);
    return () => {
      live = false;
      stop();
      pending.current?.abort();
    };
  }, []);

  const sign = useCallback(async (options: SignOptions) => {
    pending.current?.abort();
    const controller = new AbortController();
    pending.current = controller;
    setSigning(true);
    setError(undefined);
    try {
      return await signWithSignlocal({ ...options, signal: controller.signal });
    } catch (failure) {
      if (!isSignlocalError(failure)) throw failure;
      if (!isSignlocalError(failure, "UserCancelled", "Aborted")) setError(failure);
      return undefined;
    } finally {
      if (pending.current === controller) {
        pending.current = null;
        setSigning(false);
      }
    }
  }, []);

  return { status, signing, error, sign };
}
// #endregion site:react
