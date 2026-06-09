<template>
  <USelect v-model="model" :items="['VALID', 'INVALID']" />
</template>

<script setup lang="ts">
const model = defineModel<string>({ required: true });
</script>
