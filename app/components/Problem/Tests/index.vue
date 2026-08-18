<template>
  <div>
    <div class="py-2 flex justify-end gap-2">
      <UButton label="Preview Tests" variant="subtle" class="px-4" @click="" />
      <UButton label="Add Test" variant="subtle" class="px-4" @click="OnAdd" />
    </div>
    <UTable :columns="columns" :data="data">
      <template #content-cell="{ row }">
        <span v-if="row.original.testType == 'Script'">{{ row.original.content }}</span>
        <EscapedText v-else>{{ row.original.content }}</EscapedText>
      </template>
      <template #example-cell="{ row }">
        <div class="flex justify-center">
          <UCheckbox :model-value="row.original.example" disabled :color="row.original.example ? 'success' : 'neutral'"
            :ui="{
              root: 'opacity-100',
              base: `ring-1 bg-transparent cursor-default ${row.original.example ? 'ring-success' : 'ring-muted'}`,
              indicator: 'bg-transparent',
              icon: row.original.example ? 'text-success' : 'text-muted'
            }" />
        </div>
      </template>
      <template #actions-cell="{ row }">
        <div class="flex items-center gap-2">
          <UTooltip text="Delete this test">
            <UButton icon=" i-lucide-trash" color="error" variant="ghost" @click.stop="OnDelete(row.original.id)" />
          </UTooltip>
          <UTooltip text="Edit this test">
            <UButton icon="i-lucide-square-pen" color="neutral" variant="ghost" @click.stop="OnEdit(row.original)" />
          </UTooltip>
          <UTooltip text="Preview this test">
            <UButton icon="i-lucide-eye" color="neutral" variant="ghost" @click.stop="OnPreview(row.original)" />
          </UTooltip>
        </div>
      </template>
    </UTable>

    <LazyProblemTestsCreateTestModal v-model:open="createModalOpen" @success="updateTests" />
    <LazyProblemTestsEditTestModal v-model:open="editModalOpen" :test="selectedTest" @success="OnEditSuccess" />
    <LazyProblemTestsPreviewTestModal v-model:open="previewModalOpen" :test="selectedPreviewTest" />
  </div>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import type { TestDefinition } from '~/types/tests/definition';

const { invoke } = useTauri();
const { throwError } = useCustomToast();

const createModalOpen = ref(false);
const editModalOpen = ref(false);
const previewModalOpen = ref(false);
const selectedTest = ref<TestDefinition | null>(null);
const selectedPreviewTest = ref<TestDefinition | null>(null);

const data = ref<TestDefinition[]>([]);

async function OnDelete(id: number) {
  try {
    await invoke("delete_test", { id });
    await updateTests();
  } catch (e) {
    console.error(e);
    throwError("Failed to delete: " + e);
  }
}

function OnEdit(test: TestDefinition) {
  selectedTest.value = test;
  editModalOpen.value = true;
}

function OnEditSuccess(test: TestDefinition) {
  const index = data.value.findIndex(t => t.id === test.id);
  if (index !== -1) data.value[index] = test;
}

function OnAdd() {
  createModalOpen.value = true
}

function OnPreview(test: TestDefinition) {
  selectedPreviewTest.value = test;
  previewModalOpen.value = true;
}

async function updateTests() {
  try {
    const tests = await invoke<TestDefinition[]>("get_tests");
    data.value = tests.sort((a, b) => a.id - b.id);
  } catch (e) {
    console.error(e);
  }
}

onMounted(updateTests)


const columns: TableColumn<TestDefinition>[] = [
  {
    id: "id",
    header: "#",
    accessorKey: "id"
  },
  {
    id: 'content',
    header: 'Content',
  },
  {
    id: 'description',
    header: 'Desc',
    accessorKey: 'description'
  },
  {
    id: 'example',
    header: 'example',
  },
  {
    id: 'actions',
    header: ''
  }
]
</script>
