<template>
  <UModal v-model:open="isOpen">
    <UButton class="w-fit text-lg" label="Create Problem" />

    <template #content>
      <div class="overflow-y-auto max-h-[80vh]">
        <UHeader title="Create a new problem" to="" :toggle="false" />
        <UForm @submit="onSubmit" class="flex flex-col gap-4 justify-center items-center py-20">
          <UFormField label="Problem Name">
            <UInput type="text" v-model="problemName" />
          </UFormField>
          <UFormField>
            <UButton label="Project Location" color="secondary" type="button" v-if="
              problemFolder === null || problemFolder.length === 0
            " @click="onSelectPath" />
            <LazyUInput :value="problemFolder" v-else @click="onSelectPath" />
          </UFormField>

          <UButton class="w-fit text-lg px-5" type="submit" :disabled="problemName.length === 0 ||
            problemFolder === null ||
            problemFolder.length === 0
            " :loading="problems.loading">
            Create
          </UButton>
        </UForm>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";

const isOpen = ref(false);

const problemName = ref("");
const problemFolder = ref<string | null>(null);

const problems = useProblems();

async function onSelectPath() {
  problemFolder.value = await open({
    multiple: false,
    directory: true,
  });
}

async function onSubmit() {
  await problems.create(problemName.value, problemFolder.value!);

  if (problems.error === null)
    navigateTo({
      path: "/problem",
      query: {
        path: problemFolder.value! + "/" + problemName.value!,
      },
    });
  else {
    const { throwError } = useCustomToast();

    throwError(problems.error)
  }
}

function close() {
  problemFolder.value = null;
  problemName.value = "";
  isOpen.value = false;
}
</script>
