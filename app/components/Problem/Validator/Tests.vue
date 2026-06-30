<template>
  <div>
    <ProblemTestTableButtons :disabled="tableDisabled" @run="onRunAll" @add="createModalOpen = true" />
    <UTable :loading="tableDisabled" :columns="columns" :data="data"
      :meta="{
        class: {
          tr: (row: any) => row.original.actual !== '' && row.original.actual !== row.original.expected
            ? 'bg-red-50 dark:bg-red-950/30' : ''
        }
      }">

      <template #input-cell="{ row }">
        <span class="whitespace-pre-wrap">{{ row.original.input }}</span>
      </template>

      <template #actions-cell="{ row }">
        <ProblemTestTableSimpleActions @delete="onDelete(row.original.id)" @edit="onEdit(row.original)"
          @copy="onCopy(row.original)" />
      </template>
    </UTable>

    <!-- Modals -->
    <LazyProblemValidatorCreateTestModal v-model:open="createModalOpen" :copy-from="copyFromTest"
      @success="getFiles" @update:open="val => { if (!val) copyFromTest = null }" />
    <LazyProblemValidatorEditTestModal v-model:open="editModalOpen" :test="selectedTest" @success="getFiles" />
  </div>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import type { ValidatorTest, ValidatorTestError } from '~/types/validator/ValidatorTest';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { applyTestResult } from '~/utils/applyTestResult';

const { invoke, listen } = useTauri();
const { throwError } = useCustomToast();

const tableLoading = ref(false);
const testsRunning = ref(false);
const tableDisabled = computed(() => tableLoading.value || testsRunning.value);

const createModalOpen = ref(false);
const editModalOpen = ref(false);

const selectedTest = ref<ValidatorTest | null>(null);
const copyFromTest = ref<ValidatorTest | null>(null);

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

function onCopy(test: ValidatorTest) {
  copyFromTest.value = test;
  createModalOpen.value = true;
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
  unlistenResult = await listen<ValidatorTest>("validator_test_result", (e) => {
    applyTestResult(data.value, { id: e.payload.id, actual: e.payload.actual });
  });
  unlistenError = await listen<ValidatorTestError>("validator_test_error", (e) => throwError("Test " + e.payload.id + " failed with message " + e.payload.error));
});

onUnmounted(() => {
  unlistenResult?.();
  unlistenError?.();
});
</script>
