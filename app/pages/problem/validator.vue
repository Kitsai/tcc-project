<template>
  <UForm>
    <UFormField label="Language" name="language">
      <UInputMenu v-model="languageSelect" :items="languageOptions" />
    </UFormField>
  </UForm>
</template>

<script setup lang="ts">
const languageOptions = ref(['Autodetect', 'cpp.g++17', 'cpp.g++14', 'python.3']);

const languageSelect = ref("")
</script>
