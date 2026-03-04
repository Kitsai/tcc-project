import type { TauriPlugin } from "~/types/tauri";

export const useTauri = () => {
  const app = useNuxtApp();

  return app.$tauri as TauriPlugin;
}
