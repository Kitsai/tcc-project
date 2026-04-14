import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import {
  appConfigDir,
  appDataDir,
  appLocalDataDir,
  resourceDir,
} from "@tauri-apps/api/path";

export default defineNuxtPlugin((nuxtApp) => {
  const isTauri =
    typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  if (!isTauri) {
    console.warn("Not running Tauri:");
    console.log(window);
  }

  return {
    provide: {
      tauri: { invoke, listen, emit, isTauri },
    },
  };
});
