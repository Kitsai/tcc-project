<template>
  <div>
    <template v-if="props.type === 'checker'">
      <ProblemCheckerDefaultFilesSection :active-value="activeValue" @change="onActiveValueChange" />
      <UDivider />
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

const activeValue = computed(() =>
  props.type === 'checker'
    ? problems.currentProblem?.definition.checker
    : problems.currentProblem?.definition.validator
)

function onActiveValueChange(value: string | undefined) {
  if (!problems.currentProblem) return
  if (props.type === 'checker') problems.currentProblem.definition.checker = value
  else problems.currentProblem.definition.validator = value
}
</script>
