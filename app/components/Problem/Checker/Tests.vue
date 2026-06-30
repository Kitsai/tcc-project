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
        <EscapedText>{{ row.original.input }}</EscapedText>
      </template>

      <template #output-cell="{ row }">
        <EscapedText>{{ row.original.output }}</EscapedText>
      </template>

      <template #answer-cell="{ row }">
        <EscapedText>{{ row.original.answer }}</EscapedText>
      </template>

      <template #comment-cell="{ row }">
        <span class="whitespace-normal wrap-break-word max-w-sm block">
          {{ row.original.comment }}
        </span>
      </template>

      <template #actions-cell="{ row }">
        <ProblemTestTableSimpleActions
          @delete="onDelete(row.original.id)"
          @edit="onEdit(row.original)"
          @copy="onCopy(row.original)"
        />
      </template>
    </UTable>

    <!-- Modals -->
    <LazyProblemCheckerCreateTestModal v-model:open="createModalOpen" :copy-from="copyFromTest"
      @success="updateTests" @update:open="val => { if (!val) copyFromTest = null }" />
    <LazyProblemCheckerEditTestModal v-model:open="editModalOpen" :test="selectedTest" @success="updateTests" />
  </div>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import type { CheckerTest, CheckerTestError } from '~/types/checker/CheckerTest';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { applyTestResult } from '~/utils/applyTestResult';

const { invoke, listen } = useTauri();
const { throwError } = useCustomToast();

const testsLoading = ref(false);
const testsRunning = ref(false);
const tableDisabled = computed(() => testsLoading.value || testsRunning.value);

const data = ref<CheckerTest[]>([]);

const createModalOpen = ref(false);
const editModalOpen = ref(false);
const selectedTest = ref<CheckerTest | null>(null);
const copyFromTest = ref<CheckerTest | null>(null);

async function onDelete(id: number) {
  try {
    await invoke("delete_checker_test", { id });
    await updateTests();
  } catch (e) {
    console.error(e);
  }
}

function onEdit(test: CheckerTest) {
  selectedTest.value = test;
  editModalOpen.value = true;
}

function onCopy(test: CheckerTest) {
  copyFromTest.value = test;
  createModalOpen.value = true;
}

async function onRunAll() {
  testsRunning.value = true;
  try {
    await invoke("run_checker_tests");
  } catch (e) {
    throwError("Error running tests: " + e);
    console.error(e);
  }
  testsRunning.value = false;
}

async function updateTests() {
  testsLoading.value = true;
  try {
    const tests = await invoke<CheckerTest[]>("get_checker_tests");
    data.value = tests.sort((a, b) => a.id - b.id);
  } catch (e) {
    console.error(e);
  }
  testsLoading.value = false;
}

const columns: TableColumn<CheckerTest>[] = [
  {
    id: "id",
    header: "#",
    accessorKey: "id",
  },
  {
    id: "input",
    header: "Input"
  },
  {
    id: "output",
    header: "Output"
  },
  {
    id: "answer",
    header: "Answer"
  },
  {
    id: "expected",
    header: "Expected Verdict",
    accessorKey: "expected",
  },
  {
    id: "actual",
    header: "Checker Verdict",
    accessorKey: "actual"
  },
  {
    id: "comment",
    header: "Checker Comment",
  },
  {
    id: "actions",
    header: ""
  }
]

let unlistenResult: UnlistenFn;
let unlistenError: UnlistenFn;

onMounted(async () => {
  await updateTests();
  unlistenResult = await listen<CheckerTest>("checker_test_result", (e) => {
    applyTestResult(data.value, { id: e.payload.id, actual: e.payload.actual, comment: e.payload.comment });
  });
  unlistenError = await listen<CheckerTestError>("checker_test_error", (e) => throwError("Test " + e.payload.id + " failed with message " + e.payload.error));
});

onUnmounted(() => {
  unlistenResult?.();
  unlistenError?.();
});
</script>
