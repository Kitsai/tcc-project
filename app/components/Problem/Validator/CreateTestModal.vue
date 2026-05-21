<template>
  <UModal :open="props.open">
    <template #content>
      <UForm>
      </UForm>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const props = defineProps<{
  open: boolean
}>();

const emit = defineEmits<{
  success: [],
}>();

</script>
