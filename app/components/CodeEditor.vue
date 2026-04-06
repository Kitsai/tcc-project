<template>
  <div class="flex-1 w-full h-full relative overflow-hidden">
    <div ref="editorContainer" class="absolute inset-0 w-full h-full"></div>
  </div>
</template>

<script setup lang="ts">
import * as monaco from "monaco-editor"

const props = defineProps<{
  filePath: string
}>();

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

const { initLsp } = useLsp();

const setupEditor = async () => {
  await nextTick();
  if (!editorContainer.value) return;

  // Cleanup old editor and its model before creating new one
  if (editor) {
    const oldModel = editor.getModel();
    editor.dispose();
    if (oldModel) {
      oldModel.dispose();
    }
  }

  // Derive language from extension
  const ext = props.filePath.split('.').pop()?.toLowerCase();
  const languageId = ext === 'py' ? 'python' : 'cpp';

  // Create editor
  editor = monaco.editor.create(editorContainer.value, {
    value: "",
    language: languageId,
    theme: editorTheme.value,
    automaticLayout: true,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    fontSize: 14,
  });

  // Call the LSP initialization
  initLsp(props.filePath, editor);
};

onMounted(() => {
  setupEditor();
});

// Re-setup if the path changes
watch(() => props.filePath, () => {
  setupEditor();
});

onUnmounted(() => {
  if (editor) {
    const model = editor.getModel();
    editor.dispose();
    if (model) {
      model.dispose();
    }
    editor = null;
  }
});
</script>
