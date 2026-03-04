import type { invoke } from "@tauri-apps/api/core";
import type { emit, listen } from "@tauri-apps/api/event";

export interface TauriPlugin {
  invoke: typeof invoke,
  listen: typeof listen,
  emit: typeof emit,
  isTauri: boolean,
}
