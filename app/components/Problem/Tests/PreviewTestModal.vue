<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-10 overflow-y-auto flex flex-col gap-3 max-h-[70vh]">
        <h1>Preview test {{ props.test?.id }}</h1>
        <div class="overflow-auto border border-muted rounded-md p-3 min-h-24">
          <span v-if="loading" class="text-muted">Generating...</span>
          <span v-else-if="error" class="text-error">{{ error }}</span>
          <EscapedText v-else :text="content" />
        </div>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { TestDefinition } from '~/types/tests/definition';

const { invoke } = useTauri();

const open = defineModel<boolean>('open', { required: true });

const props = defineProps<{
  test: TestDefinition | null
}>();

const loading = ref(false);
const error = ref('');
const content = ref('');

watch(open, async (val) => {
  if (!val || !props.test) return;

  loading.value = true;
  error.value = '';
  content.value = '';

  try {
    content.value = await invoke<string>("preview_test", { id: props.test.id });
  } catch (e) {
    console.error(e);
    error.value = "Failed to preview test: " + e;
  } finally {
    loading.value = false;
  }
});
</script>
