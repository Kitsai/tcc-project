import { invoke } from "@tauri-apps/api/core";
import type { LanguageDetails } from "~/types/languages/details";

export const useLanguageDetails = defineStore('languageDetails', () => {
  const config = ref<LanguageDetails>(
    { gpp: false, python3: false }
  );
  const ready = ref(false);


  async function init() {
    config.value = await invoke<LanguageDetails>("check_languages");
    ready.value = true;
  }

  init();

  const allAvailable = computed(() => config.value.python3 && config.value.gpp);

  return { config, ready, allAvailable }
});
