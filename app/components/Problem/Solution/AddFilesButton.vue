<template>
  <UButton label="Add Files" type="button" variant="subtle" @click="onAddClicked" />
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
