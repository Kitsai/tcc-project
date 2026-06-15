<template>
  <div>
    <ProblemTestTableButtons :disabled="tableDisabled" />
    <UTable :loading="tableDisabled">
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

]



</script>
