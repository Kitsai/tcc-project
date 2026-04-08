import { type Problem } from "~/types/problem/problem"

export const useProblems = defineStore('problem', {
  state: () => ({
    currentProblem: null as Problem | null,
    loading: false,
    error: null as string | null,
  }),
  getters: {
    isProblemOpened: (state) => state.currentProblem !== null,
  },
  actions: {
    async load(path: string) {
      const { invoke, listen } = useTauri();
      this.loading = true;
      this.error = null;

      try {
        this.currentProblem = await invoke<Problem>('load_problem', { path });
      } catch (e) {
        this.error = e instanceof Error ? e.message : "Falha ao carregar problema";
        console.error("Falha ao carregar problema: ", e);
      }
      this.loading = false;
    },
    async create(name: string, path: string) {
      const { invoke, listen } = useTauri();
      this.loading = true;
      this.error = null;

      try {
        this.currentProblem = await invoke<Problem>('create_problem', { name, path });
      } catch (e) {
        this.error = e instanceof Error ? e.message : "Falha ao carregar problema";
        console.error("Falha ao carregar problema: ", e);
      }
      this.loading = false;
    }
  }
});
