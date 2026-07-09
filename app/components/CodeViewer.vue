<template>
  <div class="flex-1 w-full h-full relative overflow-hidden">
    <div ref="editorContainer" class="absolute inset-0 w-full h-full"></div>
  </div>
</template>

<script setup lang="ts">
import * as monaco from "monaco-editor"

const props = defineProps<{
  content: string;
  fileName: string;
}>();

const color = useColorMode();
const editorContainer = ref<HTMLElement | null>(null);
let editor: monaco.editor.IStandaloneCodeEditor | null = null;
let model: monaco.editor.ITextModel | null = null;

const editorTheme = computed<string>(() => {
  return color.value === "dark" ? "night-owl-dark" : "night-owl-light";
});

watch(editorTheme, (newTheme) => {
  if (editor) {
    monaco.editor.setTheme(newTheme);
  }
});

function languageIdFor(fileName: string) {
  const ext = fileName.split('.').pop()?.toLowerCase();
  return ext === 'py' ? 'python' : 'cpp';
}

watch([() => props.content, () => props.fileName], ([newContent, newFileName]) => {
  if (!model) return;
  model.setValue(newContent);
  monaco.editor.setModelLanguage(model, languageIdFor(newFileName));
});

onMounted(async () => {
  await nextTick();
  if (!editorContainer.value) return;

  model = monaco.editor.createModel(props.content, languageIdFor(props.fileName));

  editor = monaco.editor.create(editorContainer.value, {
    model,
    theme: editorTheme.value,
    automaticLayout: true,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    fontSize: 14,
    readOnly: true,
  });
});

onUnmounted(() => {
  if (editor) {
    editor.dispose();
    editor = null;
  }
  if (model) {
    model.dispose();
    model = null;
  }
});
</script>
