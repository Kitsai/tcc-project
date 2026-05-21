<template>
  <UTabs class="px-4" :items="tabs">

    <template #files>
      <ProblemFiles type="validator" />
    </template>

    <template #tests>
      <LazyProblemValidatorTests />
    </template>
  </UTabs>
</template>

<script setup lang="ts">
const tabs = [
  {
    label: 'Files',
    icon: 'i-lucide-file',
    slot: 'files'
  },
  {
    label: "Tests",
    icon: 'i-lucide-test-tube-diagonal',
    slot: 'tests'
  }
]
</script>
