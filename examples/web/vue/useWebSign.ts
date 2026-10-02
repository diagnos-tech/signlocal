/**
 * A Vue composable around @websign/sdk: the setup status (live), and a
 * `sign` that tracks its own progress and error. Copy it into your app.
 */

import {
  isWebSignError,
  onChange,
  status as readStatus,
  type SignOptions,
  type SignResult,
  type Status,
  sign as signWithWebSign,
  type WebSignError,
} from "@websign/sdk";
import { onMounted, onUnmounted, type Ref, ref, shallowRef } from "vue";

export interface WebSignState {
  /** undefined while the first check runs. */
  readonly status: Ref<Status | undefined>;
  readonly signing: Ref<boolean>;
  /** The last failure, except the person cancelling (nothing to show then). */
  readonly error: Ref<WebSignError | undefined>;
  /** Resolves with the result, or undefined when it failed (see `error`). */
  readonly sign: (options: SignOptions) => Promise<SignResult | undefined>;
}

export function useWebSign(): WebSignState {
  const status = shallowRef<Status>();
  const signing = ref(false);
  const error = shallowRef<WebSignError>();
  // Leaving the page section cancels a signature in progress (the app's window closes too).
  let pending: AbortController | undefined;
  let stop = () => {};

  onMounted(async () => {
    stop = onChange((s) => (status.value = s));
    status.value = await readStatus();
  });
  onUnmounted(() => {
    stop();
    pending?.abort();
  });

  async function sign(options: SignOptions): Promise<SignResult | undefined> {
    pending?.abort();
    const controller = new AbortController();
    pending = controller;
    signing.value = true;
    error.value = undefined;
    try {
      return await signWithWebSign({ ...options, signal: controller.signal });
    } catch (failure) {
      if (!isWebSignError(failure)) throw failure;
      if (!isWebSignError(failure, "UserCancelled", "Aborted")) error.value = failure;
      return undefined;
    } finally {
      if (pending === controller) {
        pending = undefined;
        signing.value = false;
      }
    }
  }

  return { status, signing, error, sign };
}
