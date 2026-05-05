import { type Problem } from "~/types/problem/problem"

export const useProblems = defineStore('problem', () => {
  const { invoke } = useTauri();

  const currentProblem = ref<Problem | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const isProblemOpened = computed(() => currentProblem.value !== null);
  const currentName = computed(() => currentProblem.value?.definition.name);

  async function load(path: string) {
    loading.value = true;
    error.value = null;

    try {
      currentProblem.value = await invoke<Problem>('load_problem', { path });
    } catch (e) {
      error.value = e instanceof Error ? e.message : "Falha ao carregar problema: " + e;
      console.error("Falha ao carregar problema: ", e);
    }
    loading.value = false;

  }

  async function create(name: string, path: string) {
    loading.value = true;
    error.value = null;

    try {
      currentProblem.value = await invoke<Problem>('create_problem', { name, path });
    } catch (e) {
      error.value = e instanceof Error ? e.message : "Falha ao criar problema: " + e;
      console.error("Falha ao carregar problema: ", e);
    }
    loading.value = false;
  }
  return {
    currentProblem,
    loading,
    error,
    isProblemOpened,
    currentName,
    load,
    create
  };
});
