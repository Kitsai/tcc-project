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
    <UFormField label="Input format:" name="input_format">
      <TexEditor v-model="state.input_format"
        class="border border-primary-200 w-full min-h-40 bg-slate-50 dark:bg-slate-950" />
    </UFormField>
    <UFormField label="Output format:" name="output_format">
      <TexEditor v-model="state.output_format"
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

    <UButton type="submit" label="Save" class="w-fit text-lg px-4" />
  </UForm>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';

const problemStore = useProblems();
const { currentProblem } = storeToRefs(problemStore);

const state = reactive({
  name: '',
  legend: '',
  input_format: '',
  output_format: '',
  notes: '',
  tutorial: ''
});

// Sync local state when a new problem is loaded into the store
watch(currentProblem, (problem) => {
  if (problem) {
    state.name = problem.stmt.name;
    state.legend = problem.stmt.legend;
    state.input_format = problem.stmt.input;
    state.output_format = problem.stmt.output;
    state.notes = problem.stmt.notes;
    state.tutorial = problem.stmt.tutorial;
  }
}, { immediate: true });

type Schema = typeof state

function onSubmit() {
  // Logic to save changes back to the store/backend would go here
}
</script>
