<template>
  <div
    class="relative border border-primary-200 dark:border-primary-800 rounded-md overflow-hidden bg-white dark:bg-zinc-950">
    <EditorContent :editor="editor" class="editor-content" />
  </div>
</template>

<script setup lang="ts">
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import 'katex/dist/katex.min.css'
import { MathematicsWithInline, migrateMathStrings } from '~/editor/extensions/mathematics';

const model = defineModel<string>();

const emit = defineEmits(['update:modelValue']);

// Helper to convert Editor JSON to raw TeX
function getTexFromEditor(editor: any) {
  if (!editor) return '';
  const json = editor.getJSON();
  let tex = '';

  const walk = (nodes: any[]) => {
    if (!nodes) return;
    nodes.forEach((node, index) => {
      if (node.type === 'text') {
        tex += node.text;
      } else if (node.type === 'inlineMath') {
        tex += `$${node.attrs.latex}$`;
      } else if (node.type === 'blockMath') {
        tex += `\n$$${node.attrs.latex}$$\n`;
      } else if (node.type === 'paragraph') {
        walk(node.content);
        if (index < nodes.length - 1) tex += '\n';
      } else if (node.content) {
        walk(node.content);
      }
    });
  };

  walk(json.content);
  return tex;
}

const editor = useEditor({
  content: model.value,
  extensions: [
    StarterKit,
    MathematicsWithInline
  ],
  onUpdate: () => {
    // We explicitly DO NOT update model.value here to prevent feedback loops while typing.
  },
  onBlur: ({ editor }) => {
    // Sync to parent when leaving the editor
    const currentTex = getTexFromEditor(editor);
    if (model.value !== currentTex) {
      model.value = currentTex;
    }
  }
})

// Debounced sync for when user is typing but hasn't blurred yet
let debounceTimer: any = null;
watch(() => editor.value?.state.doc, () => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    if (editor.value && editor.value.isFocused) {
      const currentTex = getTexFromEditor(editor.value);
      if (model.value !== currentTex) {
        model.value = currentTex;
      }
    }
  }, 1000);
});

// External updates (e.g. from store)
watch(model, (newValue) => {
  if (!editor.value) return;

  // Only update editor if it's NOT focused (meaning the change is from outside)
  if (!editor.value.isFocused) {
    const currentTex = getTexFromEditor(editor.value);
    if (newValue !== currentTex) {
      editor.value.commands.setContent(newValue || '', false);
      migrateMathStrings(editor.value);
    }
  }
});

onUnmounted(() => {
  if (debounceTimer) clearTimeout(debounceTimer);
});
</script>

<style scoped>
:deep(.ProseMirror) {
  min-height: 120px;
  padding: 0.75rem 1rem;
  outline: none;
}

:deep(.Tiptap-mathematics-render) {
  display: inline-block;
  padding: 0 0.2em;
}

:deep(.Tiptap-mathematics-editor) {
  background: #27272a;
  color: #60a5fa;
  font-family: monospace;
  border-radius: 4px;
  padding: 2px 4px;
}
</style>
