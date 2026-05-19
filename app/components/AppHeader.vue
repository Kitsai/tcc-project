<template>
  <UHeader :title="'Placeholder' + problemName" to="/" :toggle="false" class="bg-slate-200 dark:bg-slate-950">
    <template #right>
      <UColorModeButton />
      <UTooltip text="Abrir configurações" :kbds="['meta', 'Escape']">
        <UButton color="neutral" variant="ghost" to="/settings" icon="i-lucide-settings" />
      </UTooltip>
    </template>
  </UHeader>
</template>

<script setup lang="ts">

const problemStore = useProblems();
const { isProblemOpened, currentName } = storeToRefs(problemStore);

const problemName = computed(() =>
  isProblemOpened.value
    ? " - " + currentName.value
    : "",
);
</script>
