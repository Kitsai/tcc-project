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
            <UButton icon="i-lucide-trash" color="error" variant="ghost" @click.stop="onDelete" />
          </UTooltip>
          <UTooltip text="Edit this test">
            <UButton icon="i-lucide-square-pen" color="neutral" variant="ghost" @click.stop="onEdit" />
          </UTooltip>
          <UTooltip>
            <UButton icon="i-lucide-copy" color="neutral" variant="ghost" @click.stop="onCopy(row.original.input)" />
          </UTooltip>
        </div>
      </template>
    </UTable>
  </div>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import type { ValidatorTest } from '~/utils/ValidatorTest';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

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

const { throwSuccess } = useCustomToast();

function onDelete() {

}

function onEdit() {

}

async function onCopy(content: string) {
  try {
    await writeText(content);
    throwSuccess("Input copied to clipboard!");
  } catch (e) {
    console.error(e);
  }
}

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
