<template>
  <div class="flex-1 w-full h-full relative overflow-hidden">
    <div ref="editorContainer" class="absolute inset-0 w-full h-full"></div>
  </div>
</template>

<script setup lang="ts">
import * as monaco from "monaco-editor"

const props = defineProps<{
  fileName: string;
  type: string;
}>();

const { invoke } = useTauri();
const problems = useProblems();
const color = useColorMode();
const editorContainer = ref<HTMLElement | null>(null);
let editor: monaco.editor.IStandaloneCodeEditor | null = null;

const fullPath = computed(() => {
  if (!problems.currentProblem) return null;
  return `${problems.currentProblem.path}/${props.type}/${props.fileName}`;
});

const workspaceDir = computed(() => {
  if (!problems.currentProblem) return null;
  return `${problems.currentProblem.path}/${props.type}`;
});

const editorTheme = computed<string>(() => {
  return color.value === "dark" ? "night-owl-dark" : "night-owl-light";
});

watch(editorTheme, (newTheme) => {
  if (editor) {
    monaco.editor.setTheme(newTheme);
  }
});

const { initLsp } = useLsp();

const isDirty = ref(false);
let initialContent = "";

const setupEditor = async () => {
  await nextTick();
  if (!editorContainer.value || !fullPath.value || !workspaceDir.value) return;

  // Cleanup old editor and its model before creating new one
  if (editor) {
    const oldModel = editor.getModel();
    editor.dispose();
    if (oldModel) {
      oldModel.dispose();
    }
  }

  // Read file content
  try {
    initialContent = await invoke<string>("read_file_content", { path: fullPath.value });
  } catch (e) {
    console.error("Failed to read file:", e);
    initialContent = "";
  }
  isDirty.value = false;

  // Derive language from extension
  const ext = props.fileName.split('.').pop()?.toLowerCase();
  const languageId = ext === 'py' ? 'python' : 'cpp';

  // Create or reuse model with proper URI
  const pathToUri = (path: string) => {
    const normalized = path.replace(/\\/g, '/');
    return normalized.startsWith('/') ? `file://${normalized}` : `file:///${normalized}`;
  };
  
  const modelUri = monaco.Uri.parse(pathToUri(fullPath.value));
  let model = monaco.editor.getModel(modelUri);
  
  if (model) {
    model.setValue(initialContent);
    monaco.editor.setModelLanguage(model, languageId);
  } else {
    model = monaco.editor.createModel(initialContent, languageId, modelUri);
  }

  // Create editor
  editor = monaco.editor.create(editorContainer.value, {
    model: model,
    theme: editorTheme.value,
    automaticLayout: true,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    fontSize: 14,
  });

  // Handle Ctrl+S / Cmd+S inside the editor
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
    save();
  });

  editor.onDidChangeModelContent(() => {
    const currentContent = editor?.getValue();
    isDirty.value = currentContent !== initialContent;
  });

// Call the LSP initialization
  initLsp(fullPath.value, workspaceDir.value, editor);
};

const save = async () => {
  if (!editor || !fullPath.value) return;
  const content = editor.getValue();
  try {
    await invoke("write_file_content", { path: fullPath.value, content });
    initialContent = content;
    isDirty.value = false;
    console.log("File saved successfully:", fullPath.value);
  } catch (e) {
    console.error("Failed to save file:", e);
  }
};

defineExpose({
  save,
  isDirty
});

onMounted(() => {
  if (fullPath.value) {
    setupEditor();
  }
});

// Watch fullPath to re-setup if the problem loads or the file changes
watch(fullPath, (newPath) => {
  if (newPath) {
    setupEditor();
  }
}, { immediate: true });

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
