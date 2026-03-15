import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";

export default defineNuxtPlugin((nuxtApp) => {
  const isTauri =
    typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  if (!isTauri) {
    console.warn("Not running Tauri:");
    console.log(window);
  }

  nuxtApp.provide("tauri", { invoke, listen, emit, isTauri });
});
