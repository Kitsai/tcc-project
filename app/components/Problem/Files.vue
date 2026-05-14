<template>
  <div>
    <UContainer class="py-2 flex justify-end">
      <UButton type="button" label="Add File" :loading="tableLoading" @click="onAddFile" />
    </UContainer>
    <UTable v-model:row-selection="selection" :loading="tableLoading" :data="files" :columns="columns"
      :table-options="{ enableMultiRowSelection: false }"
      :ui="{ tr: 'border-l-4 border-transparent transition-all cursor-pointer' }" :meta="{
        class: {
          tr: (row: any) => selection[row.id] ? '!bg-primary-50 dark:!bg-primary-950/50 !border-l-primary-500' : ''
        }
      }" @select="onSelect">
      <template #fileName-cell="{ row }">
        <div class="flex items-center gap-2">
          <UIcon v-if="selection[row.id]" name="i-lucide-check-circle-2" class="size-4 text-primary-500" />
          <span :class="{ 'font-bold text-primary-600 dark:text-primary-400': selection[row.id] }">
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
  type: 'validators' | 'checkers'
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

const tableLoading = ref(true);
const files = ref<string[]>([]);

const selection = ref<Record<string, boolean>>({});

const { invoke } = useTauri();
const { throwError } = useCustomToast();
const problems = useProblems();
const router = useRouter();

async function getFiles() {
  console.log("[Files.vue] getFiles called for type:", props.type);
  tableLoading.value = true;

  try {
    const fetchedFiles = await invoke<string[]>("get_files_from", { dir: props.type })
    console.log("[Files.vue] fetchedFiles:", fetchedFiles);
    files.value = fetchedFiles;
    applyAutoSelection();
  } catch (e) {
    console.error("[Files.vue] Error in getFiles:", e);
  }

  tableLoading.value = false;
}

function applyAutoSelection() {
  if (problems.currentProblem) {
    const savedFileName = props.type === "validators"
      ? problems.currentProblem.definition.validator
      : problems.currentProblem.definition.checker;

    console.log("[Files.vue] Attempting auto-select. Saved:", savedFileName);

    if (savedFileName) {
      const index = files.value.findIndex(f => f === savedFileName);
      console.log("[Files.vue] Found index:", index);
      if (index !== -1) {
        selection.value = { [index.toString()]: true };
        console.log("[Files.vue] selection.value updated:", selection.value);
      }
    }
  } else {
    console.log("[Files.vue] currentProblem is not loaded yet");
  }
}

async function onSelect(_: Event, row: TableRow<string>) {
  selection.value = { [row.id]: true };

  try {
    await invoke("select_problem_file", { dir: props.type, file: row.original });

    if (problems.currentProblem) {
      if (props.type === "checkers") problems.currentProblem.definition.checker = row.original;
      else problems.currentProblem.definition.validator = row.original;
    }
  } catch (e) {
    selection.value = {};
    throwError("Failed to select file");
  }
}


function onAddFile() {
  console.log("Add file clicked");
}

function onEdit(filename: string) {
  router.push({
    path: '/problem/editor',
    query: {
      type: props.type,
      file: filename
    }
  });
}

onMounted(() => {
  getFiles();
});
</script>
