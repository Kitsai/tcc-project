<template>
  <UModal v-model:open="open" :ui="{ content: 'max-w-5xl w-[90vw]' }">
    <template #content>
      <div class="p-6 h-[80vh] flex flex-col gap-3">
        <h1 class="text-lg font-semibold text-highlighted">{{ fileName }}</h1>
        <CodeViewer class="flex-1" :content="content" :file-name="fileName ?? ''" />
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const { invoke } = useTauri();
const { throwError } = useCustomToast();

const open = defineModel<boolean>('open', { required: true });

const props = defineProps<{
  fileName: string | null
}>();

const content = ref("");

watch(open, async (val) => {
  if (val && props.fileName) {
    try {
      content.value = await invoke<string>("read_default_checker_content", { name: props.fileName });
    } catch (e) {
      throwError("Failed to read file: " + e);
      content.value = "";
    }
  } else {
    content.value = "";
  }
});
</script>
