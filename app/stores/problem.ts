import { type Problem } from "~/types/problem/problem"
import { type PolygonImportResult } from "~/types/polygon/import"
import { type ImportFormat } from "~/types/problem/import_format"

export const useProblems = defineStore('problem', () => {
  const { invoke } = useTauri();

  const currentProblem = ref<Problem | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const importWarnings = ref<string[]>([]);

  const isProblemOpened = computed(() => currentProblem.value !== null);
  const currentName = computed(() => currentProblem.value?.definition.name);

  async function load(path: string) {
    loading.value = true;
    error.value = null;

    try {
      currentProblem.value = await invoke<Problem>('load_problem', { path });
    } catch (e) {
      error.value = e instanceof Error ? e.message : "Failed to load problem: " + e;
      console.error("Failed to load problem: ", e);
    }
    loading.value = false;

  }

  async function create(name: string, path: string) {
    loading.value = true;
    error.value = null;

    try {
      currentProblem.value = await invoke<Problem>('create_problem', { name, path });
    } catch (e) {
      error.value = e instanceof Error ? e.message : "Failed to create problem: " + e;
      console.error("Failed to create problem: ", e);
    }
    loading.value = false;
  }

  async function importPackage(format: ImportFormat, source: string, destParent: string, name: string) {
    loading.value = true;
    error.value = null;
    importWarnings.value = [];

    try {
      const result = await invoke<PolygonImportResult>(format === 'moj' ? 'import_moj_problem' : 'import_polygon_problem', { source, destParent, name });
      currentProblem.value = result.problem;
      importWarnings.value = result.warnings;
    } catch (e) {
      error.value = e instanceof Error ? e.message : "Failed to import problem: " + e;
      console.error("Failed to import problem: ", e);
    }
    loading.value = false;
  }

  function clearImportWarnings() {
    importWarnings.value = [];
  }

  return {
    currentProblem,
    loading,
    error,
    importWarnings,
    isProblemOpened,
    currentName,
    load,
    create,
    importPackage,
    clearImportWarnings
  };
});
