<template>
  <span class="whitespace-pre-wrap">
    <slot>{{ text }}</slot>
  </span>
</template>

<script setup lang="ts">
const props = defineProps<{
  text?: string
}>();
</script>
