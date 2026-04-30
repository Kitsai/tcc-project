<template>
  <UForm :state="state" class="gap-y-4 flex flex-col" @submit="onSubmit">
    <UFormField label="Name:" name="name">
      <UInput v-model="state.name"
        class="border border-primary-200 dark:border-primary-800 rounded-md bg-slate-50 dark:bg-slate-950" />
    </UFormField>
    <UFormField label="Legend:" name="legend">
      <TexEditor v-model="state.legend"
        class="border border-primary-200 w-full min-h-40 bg-slate-50 dark:bg-slate-950" />
    </UFormField>
    <UFormField label="Input format:" name="input">
      <TexEditor v-model="state.input"
        class="border border-primary-200 w-full min-h-40 bg-slate-50 dark:bg-slate-950" />
    </UFormField>
    <UFormField label="Output format:" name="output">
      <TexEditor v-model="state.output"
        class="border border-primary-200 w-full min-h-40 bg-slate-50 dark:bg-slate-950" />
    </UFormField>
    <UFormField label="Notes:" name="notes">
      <TexEditor v-model="state.notes"
        class="border border-primary-200 w-full min-h-40 bg-slate-50 dark:bg-slate-950" />
    </UFormField>
    <UFormField label="Tutorial:" name="tutorial">
      <TexEditor v-model="state.tutorial"
        class="border border-primary-200 w-full min-h-40 bg-slate-50 dark:bg-slate-950" />
    </UFormField>

    <UButton type="submit" label="Save" :loading="loading" class="w-fit text-lg px-4" />
  </UForm>
</template>

<script setup lang="ts">
import type { ProblemStatement } from '~/types/problem/problem';

const problemStore = useProblems();
const { currentProblem } = storeToRefs(problemStore);

const loading = ref(false);

const state = reactive<ProblemStatement>({
  name: '',
  legend: '',
  input: '',
  output: '',
  notes: '',
  tutorial: ''
});

// Sync local state when a new problem is loaded into the store
watch(currentProblem, (problem) => {
  if (problem) {
    state.name = problem.stmt.name;
    state.legend = problem.stmt.legend;
    state.input = problem.stmt.input;
    state.output = problem.stmt.output;
    state.notes = problem.stmt.notes;
    state.tutorial = problem.stmt.tutorial;
  }
}, { immediate: true });

async function onSubmit() {
  const { invoke } = useTauri();
  const { throwSuccess, throwError } = useCustomToast();

  loading.value = true;
  try {
    await invoke("save_statement", { stmt: { ...state } });

    if (currentProblem.value) {
      currentProblem.value.stmt = { ...state };
    }

    throwSuccess('Problem statement saved successfully');
  } catch (e) {
    throwError(String(e));
  } finally {
    loading.value = false;
  }
}
</script>
