<template>
  <USelect
    v-model="model"
    :items="['OK', 'WRONG_ANSWER', 'PRESENTATION_ERROR', 'CRASHED']"
    class="w-56"
  />
</template>

<script setup lang="ts">
const model = defineModel<string>({ required: true });
</script>
