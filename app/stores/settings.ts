import type { Settings } from "~/types/settings";

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    settings: null as Settings | null,
    loading: false,
    error: null as string | null,
  }),

  getters: {
    isConfigured: (state) => state.settings !== null,
    maxConcurrent: (state) => state.settings?.max_concurrency ?? 0,
  },

  actions: {
    async load() {
      this.loading = true;
      this.error = null;

      try {
        const { invoke } = useTauri();
        this.settings = await invoke<Settings>('get_settings');
      } catch (e) {
        this.error = e instanceof Error ? e.message : 'Failed to load settings'
        console.error('Failed to load settings: ', e);
      } finally {
        this.loading = false;
      }
    },
    async save(newSettings: Settings) {
      this.loading = true;
      this.error = null;

      try {
        const { invoke } = useTauri();
        this.settings = await invoke<Settings>('save_settings', { newSettings });
      } catch (e) {
        this.error = e instanceof Error ? e.message : 'Failed to save settings'
        console.error('Failed to save settings: ', e);
      } finally {
        this.loading = false;
      }
    }
  }
});
