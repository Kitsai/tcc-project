import type { MaybePromise } from "~/utils/MaybePromise";

export const useWindowFocus = (callback: () => MaybePromise<void>, cooldownMs = 5000) => {
  let lastFetch = 0;

  const guardedFetch = () => {
    const now = Date.now();

    if (now - lastFetch < cooldownMs) return;
    lastFetch = now;
    callback()
  }

  const addWindowFocus = () => {
    window.addEventListener('focus', guardedFetch);
  }

  const removeWindowFocus = () => {
    window.removeEventListener('focus', guardedFetch);
  }
}
