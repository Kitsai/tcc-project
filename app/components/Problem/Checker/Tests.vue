<template>
  <div>
    <ProblemTestTableButtons :disabled="tableDisabled" />
    <UTable :loading="tableDisabled" :columns="columns" :data="data">
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
        <ProblemTestTableSimpleActions />
      </template>
    </UTable>
  </div>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import type { CheckerTest } from '~/types/checker/CheckerTest';

const testsLoading = ref(false);
const testsRunning = ref(false);
const tableDisabled = computed(() => testsLoading.value || testsRunning.value);

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

const data: CheckerTest[] = [
  {
    id: 1,
    input: "1\n1\n1\n",
    output: "nice",
    answer: "yes",
    expected: "PRESENTATION_ERROR",
    actual: "PRESENTATION_ERROR",
    comment: "wrong output format Expected YES or NO token, but found \"NICE\" (test case 1)"
  }
]

</script>
