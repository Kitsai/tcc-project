<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-10 overflow-y-auto flex flex-col gap-3 max-h-[70vh]">
        <h1>Preview test {{ props.test?.id }}</h1>

        <span v-if="loading" class="text-muted">Generating...</span>
        <span v-else-if="error" class="text-error">{{ error }}</span>

        <div v-else-if="outcome?.kind === 'Single'" class="overflow-auto border border-muted rounded-md p-3 min-h-24">
          <EscapedText :text="outcome.content" />
        </div>

        <template v-else-if="outcome?.kind === 'Multiple'">
          <div class="flex items-center justify-between gap-2">
            <span class="text-muted text-sm">
              This generator produced {{ outcome.files.length }} test files in one run.
            </span>
            <UButton
              label="Import all as tests"
              icon="i-lucide-download"
              :loading="importing"
              @click="onImportAll"
            />
          </div>
          <div v-for="file in outcome.files" :key="file.name" class="flex flex-col gap-1">
            <span class="text-sm font-medium text-highlighted">{{ file.name }}</span>
            <div class="overflow-auto border border-muted rounded-md p-3 min-h-16">
              <EscapedText :text="file.content" />
            </div>
          </div>
        </template>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { GeneratedFile, PreviewOutcome, TestDefinition } from '~/types/tests/definition';

const { invoke } = useTauri();
const { throwError, throwSuccess } = useCustomToast();

const open = defineModel<boolean>('open', { required: true });

const props = defineProps<{
  test: TestDefinition | null
}>();

const emit = defineEmits<{
  imported: [],
}>();

const loading = ref(false);
const importing = ref(false);
const error = ref('');
const outcome = ref<PreviewOutcome | null>(null);

watch(open, async (val) => {
  if (!val || !props.test) return;

  loading.value = true;
  error.value = '';
  outcome.value = null;

  try {
    outcome.value = await invoke<PreviewOutcome>("preview_test", { id: props.test.id });
  } catch (e) {
    console.error(e);
    error.value = "Failed to preview test: " + e;
  } finally {
    loading.value = false;
  }
});

async function onImportAll() {
  if (outcome.value?.kind !== 'Multiple') return;

  const files: GeneratedFile[] = outcome.value.files;
  importing.value = true;
  try {
    await invoke("import_generated_tests", { files });
    throwSuccess(`Imported ${files.length} tests`);
    emit('imported');
    open.value = false;
  } catch (e) {
    console.error(e);
    throwError("Failed to import tests: " + e);
  } finally {
    importing.value = false;
  }
}
</script>
