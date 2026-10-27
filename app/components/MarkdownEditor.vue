<template>
  <div
    class="relative border border-primary-200 dark:border-primary-800 rounded-md overflow-hidden bg-white dark:bg-zinc-950">
    <EditorContent :editor="editor" class="editor-content" />
  </div>
</template>

<script setup lang="ts">
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import { Markdown } from '@tiptap/markdown'
import 'katex/dist/katex.min.css'
import { MathematicsWithInline, migrateMathStrings } from '~/editor/extensions/mathematics';

const model = defineModel<string>();

const emit = defineEmits(['update:modelValue']);

const editor = useEditor({
  content: model.value,
  contentType: 'markdown',
  extensions: [
    StarterKit,
    Markdown,
    MathematicsWithInline
  ],
  onUpdate: () => {
    // We explicitly DO NOT update model.value here to prevent feedback loops while typing.
  },
  onBlur: ({ editor }) => {
    // Sync to parent when leaving the editor
    const currentMarkdown = editor.getMarkdown();
    if (model.value !== currentMarkdown) {
      model.value = currentMarkdown;
    }
  }
})

// Debounced sync for when user is typing but hasn't blurred yet
let debounceTimer: any = null;
watch(() => editor.value?.state.doc, () => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    if (editor.value && editor.value.isFocused) {
      const currentMarkdown = editor.value.getMarkdown();
      if (model.value !== currentMarkdown) {
        model.value = currentMarkdown;
      }
    }
  }, 1000);
});

// External updates (e.g. from store)
watch(model, (newValue) => {
  if (!editor.value) return;

  // Only update editor if it's NOT focused (meaning the change is from outside)
  if (!editor.value.isFocused) {
    const currentMarkdown = editor.value.getMarkdown();
    if (newValue !== currentMarkdown) {
      editor.value.commands.setContent(newValue || '', { emitUpdate: false, contentType: 'markdown' });
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
