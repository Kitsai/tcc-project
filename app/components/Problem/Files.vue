<template>
  <div>
    <template v-if="props.type === 'checker'">
      <ProblemCheckerDefaultFilesSection :active-value="activeValue" @change="onActiveValueChange" />
      <USeparator />
    </template>

    <ProblemProjectFilesSection :type="props.type" :active-value="activeValue" @change="onActiveValueChange" />
  </div>
</template>

<script setup lang="ts">
import type { ProblemFileTypes } from '~/types/problem/files';

const props = defineProps<{
  type: ProblemFileTypes
}>()

const problems = useProblems()

const activeValue = computed(() => {
  if (props.type === 'checker') return problems.currentProblem?.definition.checker
  if (props.type === 'validator') return problems.currentProblem?.definition.validator
  return undefined
})

function onActiveValueChange(value: string | undefined) {
  if (!problems.currentProblem) return
  if (props.type === 'checker') problems.currentProblem.definition.checker = value
  else if (props.type === 'validator') problems.currentProblem.definition.validator = value
}
</script>
