import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";

export default defineNuxtPlugin((nuxtApp) => {
  const isTauri = typeof window !== "undefined" && "__TAURI__" in window;
  if (!isTauri) {
    console.warn("Not running Tauri:");
    console.info(window);
  }

  nuxtApp.provide("tauri", { invoke, listen, emit, isTauri });
});
