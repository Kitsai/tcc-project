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
import { MathematicsWithInline } from '~/editor/extensions/mathematics';

const model = defineModel<string>("");

const emit = defineEmits(['update:modelValue']);

const editor = useEditor({
  content: model.value,
  extensions: [
    StarterKit,
    MathematicsWithInline
  ],
  onUpdate: ({ editor }) => {
    model.value = editor.getHTML()
  }
})
</script>

<style scoped>
/* Ensure the editor area is clickable and has some breathing room */
:deep(.ProseMirror) {
  min-height: 120px;
  padding: 0.75rem 1rem;
  outline: none;
}

/* Tiptap specific math classes */
:deep(.Tiptap-mathematics-render) {
  display: inline-block;
  padding: 0 0.2em;
}

:deep(.Tiptap-mathematics-editor) {
  background: #27272a;
  /* matches your dark theme */
  color: #60a5fa;
  font-family: monospace;
  border-radius: 4px;
  padding: 2px 4px;
}
</style>
