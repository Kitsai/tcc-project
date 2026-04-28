<template>
  <div class="grow flex flex-col h-full overflow-hidden">
    <UNavigationMenu :items="navItems" orientation="horizontal"
      class="px-4 border-b border-gray-200 dark:border-gray-800 justify-end" />

    <div class="grow overflow-auto p-4">
      <NuxtPage />
    </div>
  </div>
</template>

<script setup lang="ts">
const route = useRoute();
const problems = useProblems();

const navItems = [
  {
    label: 'Statement', to: { path: '/problem' }, exact: true
  },
  {
    label: 'Checker', to: { path: '/problem/checker' }
  },
];
</script>
