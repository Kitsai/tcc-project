<template>
  <div class="py-2 flex justify-end gap-2">
    <UButton label="Run Tests" variant="subtle" class="px-4" :disabled="props.disabled" @click="emit('run')" />
    <UButton label="Add Test" variant="subtle" class="px-4" @click="emit('add')" />
  </div>
</template>

<script setup lang="ts">
const emit = defineEmits<{
  run: [],
  add: [],
}>();
const props = defineProps<{
  disabled: boolean
}>();
</script>
