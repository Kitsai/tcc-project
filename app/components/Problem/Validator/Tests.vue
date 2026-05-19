<template>
  <div>
    <div class="py-2 flex justify-end gap-2">
      <UButton label="Run Tests" class="px-4" />
      <UButton label="Add Test" class="px-4" />
    </div>
    <UTable :loading="tableLoading" :columns="columns" :data="data">

      <template #input-cell="{ row }">
        <span class="whitespace-pre-wrap">{{ row.original.input }}</span>
      </template>

      <template #actions-cell="{ row }">
        <div class="flex items-center gap-2">
          <UTooltip text="Delete this test">
            <UButton icon="i-lucide-trash" color="error" variant="ghost" />
          </UTooltip>
          <UTooltip text="Edit this test">
            <UButton icon="i-lucide-square-pen" color="neutral" variant="ghost" />
          </UTooltip>
          <UTooltip>
            <UButton icon="i-lucide-copy" color="neutral" variant="ghost" />
          </UTooltip>
        </div>
      </template>
    </UTable>
  </div>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import type { ValidatorTest } from '~/utils/ValidatorTest';

const tableLoading = ref(false);
const columns: TableColumn<ValidatorTest>[] = [
  {
    id: "id",
    header: "#",
    accessorKey: "id",
  },
  {
    id: "input",
    header: "Input",
  },
  {
    id: "expected",
    header: "Expected verdict",
    accessorKey: "expected",
  },
  {
    id: "actual",
    header: "Validator verdict",
    accessorKey: "actual",
  },
  {
    id: "actions",
    header: "",
  }
]

const data = ref<ValidatorTest[]>([
  {
    id: 1,
    input: "3\n1 2 3\n",
    expected: "VALID",
    actual: "VALID",
  },
  {
    id: 2,
    input: "3\n1 2 \n",
    expected: "INVALID",
    actual: "INVALID",
  },
  {
    id: 3,
    input: "2\n 1 2\n",
    expected: "VALID",
    actual: ""
  }
]);

</script>
