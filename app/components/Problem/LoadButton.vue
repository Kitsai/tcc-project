<template>
  <UButton class="w-fit text-lg" label="Abrir Problema" @click="onClick" />
</template>

<script setup lang="ts">

import { open } from "@tauri-apps/plugin-dialog";

const problems = useProblems();

async function onClick() {
  const path = await open({
    multiple: false,
  });

  if (path != null) {
    await problems.load(path);
    if (problems.error === null) navigateTo({
      path: "/problem",
      query: {
        path: path
      }
    });
    else {
      const throwError = useErrorToast();

      throwError(problems.error);
    }
  }
}
</script>
