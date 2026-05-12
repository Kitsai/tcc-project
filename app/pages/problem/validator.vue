<template>
  <div class="flex flex-col gap-5">
    <UFormField label="Language" name="language">
      <UInputMenu v-model="languageSelect" :items="languageOptions" />
    </UFormField>
    <UTabs :items="tabs">
      <template #files>
        <ProblemFiles type="validators" />
      </template>
      <template #tests>
      </template>
    </UTabs>
  </div>
</template>

<script setup lang="ts">
const languageOptions = ref(['Autodetect', 'cpp.g++17', 'cpp.g++14', 'python.3']);

const languageSelect = ref("Autodetect");

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
