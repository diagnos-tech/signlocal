/**
 * A Vue composable around @signlocal/sdk: the setup status (live), and a
 * `sign` that tracks its own progress and error. Copy it into your app.
 */

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
import { onMounted, onUnmounted, type Ref, ref, shallowRef } from "vue";

export interface SignlocalState {
  /** undefined while the first check runs. */
  readonly status: Ref<Status | undefined>;
  readonly signing: Ref<boolean>;
  /** The last failure, except the person cancelling (nothing to show then). */
  readonly error: Ref<SignlocalError | undefined>;
  /** Resolves with the result, or undefined when it failed (see `error`). */
  readonly sign: (options: SignOptions) => Promise<SignResult | undefined>;
}

export function useSignlocal(): SignlocalState {
  const status = shallowRef<Status>();
  const signing = ref(false);
  const error = shallowRef<SignlocalError>();
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
      return await signWithSignlocal({ ...options, signal: controller.signal });
    } catch (failure) {
      if (!isSignlocalError(failure)) throw failure;
      if (!isSignlocalError(failure, "UserCancelled", "Aborted")) error.value = failure;
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
