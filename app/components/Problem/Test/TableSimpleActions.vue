<template>
  <div class="flex items-center gap-2">
    <UTooltip text="Delete this test">
      <UButton icon="i-lucide-trash" color="error" variant="ghost" @click.stop="emit('delete')" />
    </UTooltip>
    <UTooltip text="Edit this test">
      <UButton icon="i-lucide-square-pen" color="neutral" variant="ghost" @click.stop="emit('edit')" />
    </UTooltip>
    <UTooltip text="Copy this test">
      <UButton icon="i-lucide-copy" color="neutral" variant="ghost" @click.stop="emit('copy')" />
    </UTooltip>
  </div>
</template>

<script setup lang="ts">
const emit = defineEmits<{
  delete: [],
  edit: [],
  copy: [],
}>();
</script>
