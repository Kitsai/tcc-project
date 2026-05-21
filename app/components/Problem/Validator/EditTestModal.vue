<template>
  <UModal :open="props.open">
    <template #content>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const props = defineProps<{
  open: boolean
}>();
</script>
