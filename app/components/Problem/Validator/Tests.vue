<template>
  <div>
    <div class="py-2 flex justify-end gap-2">
      <UButton label="Run Tests" variant="subtle" class="px-4" :disabled="tableLoading || testsRunning"
        @click="onRunAll" />
      <UButton label="Add Test" variant="subtle" class="px-4" @click="createModalOpen = true" />
    </div>
    <UTable :loading="tableLoading || testsRunning" :columns="columns" :data="data">

      <template #input-cell="{ row }">
        <span class="whitespace-pre-wrap">{{ row.original.input }}</span>
      </template>

      <template #actions-cell="{ row }">
        <div class="flex items-center gap-2">
          <UTooltip text="Delete this test">
            <UButton icon="i-lucide-trash" color="error" variant="ghost" @click.stop="onDelete(row.original.id)" />
          </UTooltip>
          <UTooltip text="Edit this test">
            <UButton icon="i-lucide-square-pen" color="neutral" variant="ghost" @click.stop="onEdit(row.original)" />
          </UTooltip>
          <UTooltip>
            <UButton icon="i-lucide-copy" color="neutral" variant="ghost" @click.stop="onCopy(row.original.input)" />
          </UTooltip>
        </div>
      </template>
    </UTable>

    <!-- Modals -->
    <LazyProblemValidatorCreateTestModal v-model:open="createModalOpen" @success="getFiles" />
    <LazyProblemValidatorEditTestModal v-model:open="editModalOpen" :test="selectedTest" @success="getFiles" />
  </div>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import type { ValidatorTest, ValidatorTestError } from '~/utils/ValidatorTest';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import type { UnlistenFn } from '@tauri-apps/api/event';

const { invoke, listen } = useTauri();
const { throwSuccess, throwError } = useCustomToast();

const tableLoading = ref(false);
const testsRunning = ref(false);
const createModalOpen = ref(false);
const editModalOpen = ref(false);

const selectedTest = ref<ValidatorTest | null>(null)

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


async function onDelete(id: number) {
  try {
    await invoke("delete_validator_test", { id });
    await getFiles();
  } catch (e) {
    console.error(e);
  }
}

function onEdit(test: ValidatorTest) {
  selectedTest.value = test;
  editModalOpen.value = true;
}

async function onCopy(content: string) {
  try {
    await writeText(content);
    throwSuccess("Input copied to clipboard!");
  } catch (e) {
    console.error(e);
  }
}

async function onRunAll() {
  testsRunning.value = true;
  try {
    await invoke("run_validator_tests");
  } catch (e) {
    throwError("Error running tests: " + e);
    console.error(e);
  }
  testsRunning.value = false;
}

const data = ref<ValidatorTest[]>([]);

async function getFiles() {
  tableLoading.value = true;

  try {
    const tests = await invoke<ValidatorTest[]>("get_validator_tests");
    data.value = tests.sort((a, b) => a.id - b.id);
  } catch (e) {
    console.error(e);
  }

  tableLoading.value = false;
}


let unlistenResult: UnlistenFn;
let unlistenError: UnlistenFn;

onMounted(async () => {
  await getFiles();
  unlistenResult = await listen("validator_test_result", getFiles);
  unlistenError = await listen<ValidatorTestError>("validator_test_error", (e) => throwError("Test " + e.payload.id + " failed with message " + e.payload.error));
});

onUnmounted(() => {
  unlistenResult?.();
  unlistenError?.();
});
</script>
