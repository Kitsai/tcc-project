<template>
  <div>
    <UContainer class="py-2 flex justify-end">
      <UButton type="button" label="Add File" :loading="tableLoading" @click="onAddFile" />
    </UContainer>
    <UTable :loading="tableLoading" :data="files" :columns="columns" :table-options="{ enableMultiRowSelection: false }"
      @select="onSelect" :ui="{ tr: 'border-l-4 border-transparent transition-all cursor-pointer' }" :meta="{
        class: {
          tr: (row: any) => selected?.id === row.id ? '!bg-primary-50 dark:!bg-primary-950/50 !border-l-primary-500' : ''
        }
      }">
      <template #fileName-cell="{ row }">
        <div class="flex items-center gap-2">
          <UIcon v-if="selected?.id === row.id" name="i-lucide-check-circle-2" class="size-4 text-primary-500" />
          <span :class="{ 'font-bold text-primary-600 dark:text-primary-400': selected?.id === row.id }">
            {{ row.original }}
          </span>
        </div>
      </template>

      <template #actions-cell="{ row }">
        <div class="flex justify-end">
          <UTooltip text="Edit the file">
            <UButton icon="i-lucide-square-pen" color="neutral" variant="ghost" @click.stop="onEdit(row.original)" />
          </UTooltip>
        </div>
      </template>
    </UTable>
  </div>
</template>

<script setup lang="ts">
import type { TableColumn, TableRow } from '@nuxt/ui';

const props = defineProps<{
  type: 'validators' | 'checkers' | 'generators' | 'solutions'
}>();

const columns: TableColumn<string>[] = [
  {
    id: 'fileName',
    header: "File Name",
    accessorFn: (row: string) => row
  },
  {
    id: 'actions',
    header: ""
  }
]

const tableLoading = ref(false);

const files = ref<string[]>([]);

const selected = ref<TableRow<string> | null>(null);

const { invoke } = useTauri();

async function getFiles() {
  tableLoading.value = true;

  try {
    files.value = await invoke<string[]>("get_files_from", { dir: props.type })
  } catch (e) {
    console.error(e);
  }

  tableLoading.value = false;
}

function onSelect(e: Event, row: TableRow<string>) {
  if (selected.value) selected.value.toggleSelected(false);
  row.toggleSelected(true);
  selected.value = row;
}


function onAddFile() {
  console.log("Add file clicked");
}

function onEdit(filename: string) {
  console.log("Editing file " + filename);
}

getFiles();


</script>
