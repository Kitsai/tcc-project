<template>
  <div class="flex-1 w-full flex flex-col overflow-hidden">
    <div
      class="flex items-center px-4 py-2 border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-900 gap-4">
      <USelect v-model="selectedExt" :items="languages" size="sm" class="w-40" />
      <span class="text-xs text-gray-500 font-mono">File: {{ currentFilePath }}</span>
    </div>
    <CodeEditor :file-path="currentFilePath" />
  </div>
</template>

<script setup lang="ts">
import type { SelectItem } from '@nuxt/ui';

const languages = ref<SelectItem[]>([
  { label: 'C++ (.cpp)', value: 'cpp' },
  { label: 'Python (.py)', value: 'py' }
]);
const selectedExt = ref('cpp');

const currentFilePath = computed(() => {
  const root = "/home/Kitsai/coding/tcc/tcc-project";
  return selectedExt.value === 'py' ? `${root}/test.py` : `${root}/test.cpp`;
});
</script>
