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
            <UButton icon="i-lucide-trash" color="error" variant="ghost" @click.stop="OnDelete(row.original.id)" />
          </UTooltip>
          <UTooltip text="Edit this test">
            <UButton icon="i-lucide-square-pen" color="neutral" variant="ghost"
              @click.stop="OnEdit(row.original)" />
          </UTooltip>
          <UTooltip text="Preview this test">
            <UButton icon="i-lucide-eye" color="neutral" variant="ghost" @click.stop="OnDelete(row.original.id)" />
          </UTooltip>
        </div>
      </template>
    </UTable>

    <LazyProblemTestsCreateTestModal v-model:open="createModalOpen"/>
    <LazyProblemTestsEditTestModal v-model:open="editModalOpen" :test="selectedTest" @success="OnEditSuccess" />
  </div>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import type { TestDefinition } from '~/types/tests/definition';

const createModalOpen = ref(false);
const editModalOpen = ref(false);
const selectedTest = ref<TestDefinition | null>(null);

async function OnDelete(id: number) {

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

const data = ref<TestDefinition[]>([
  {
    id: 1,
    content: "1\n1\n1",
    description: "basic test for example",
    example: true,
    testType: 'Manual'
  },
  {
    id: 2,
    content: "generator",
    description: "",
    example: false,
    testType: 'Script'
  },
])
</script>
