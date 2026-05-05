<template>
  <UContainer class="flex flex-col gap-4 justify-center items-center h-[80vh]">
    <LanguageAlert />
    <ProblemLoadButton />
    <ProblemCreateModal />
  </UContainer>
</template>

<script setup lang="ts">
import removeProblem from "~/middleware/removeProblem";

definePageMeta({
  middleware: [removeProblem],
});
</script>
