<template>
  <div class="flex justify-end items-center min-h-10">
    <UButton label="Add Files" type="button" variant="subtle" @click="onAddClicked" />
  </div>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog';

const { invoke } = useTauri();
const { throwError } = useCustomToast();

const emit = defineEmits<{
  filesAdded: []
}>();

async function onAddClicked() {
  const paths = await open({
    multiple: true,
  });

  try {
    await invoke("add_solution_files", { paths })
    emit("filesAdded");
  } catch (e) {
    console.error(e);
    throwError("Failed to add files");
  }
}
</script>
