<template>
  <div class="flex-1 w-full relative overflow-hidden">
    <MonacoEditor v-model="value" lang="cpp" class="absolute inset-0" :options="{
      theme: editorTheme,
      automaticLayout: true,
      minimap: { enabled: false }
    }" />
  </div>
</template>

<script setup lang="ts">
const value = ref('');
const color = useColorMode();
const error = ref<string | null>(null);

const editorTheme = computed<string>(() => {
  return color.value === "dark" ? "night-owl-dark" : "night-owl-light";
});
</script>
