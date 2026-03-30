<template>
  <div class="flex-1 w-full relative overflow-hidden">
    <div ref="editorContainer" class="absolute inset-0"></div>
  </div>
</template>

<script setup lang="ts">
import * as monaco from "monaco-editor"

const color = useColorMode();
const editorContainer = ref<HTMLElement | null>(null);
let editor: monaco.editor.IStandaloneCodeEditor | null = null;

const editorTheme = computed<string>(() => {
  return color.value === "dark" ? "night-owl-dark" : "night-owl-light";
});

watch(editorTheme, (newTheme) => {
  if (editor) {
    monaco.editor.setTheme(newTheme);
  }
});

onMounted(() => {
  if (!editorContainer.value) return;

  // Create editor
  editor = monaco.editor.create(editorContainer.value, {
    value: "",
    language: 'cpp',
    theme: editorTheme.value,
    automaticLayout: true,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    fontSize: 14,
  });
});

onUnmounted(() => {
  if (editor) {
    editor.dispose();
    editor = null;
  }
});
</script>
