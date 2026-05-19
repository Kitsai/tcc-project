<template>
  <div class="fixed inset-0 bg-default z-50 flex flex-col">
    <div class="h-14 border-b border-gray-200 dark:border-gray-800 flex items-center px-4 justify-between bg-default">
      <div class="flex items-center gap-3">
        <UButton icon="i-lucide-arrow-left" variant="ghost" color="neutral" @click="router.back()" />
        <span class="font-medium text-sm truncate">
          {{ route.query.file }}{{ editorRef?.isDirty ? '*' : '' }}
        </span>
      </div>
      <div class="flex items-center gap-2">
        <UTooltip text="Save (Meta+S)">
          <UButton label="Save" color="primary" :disabled="!editorRef?.isDirty" @click="onSave" />
        </UTooltip>
      </div>
    </div>


    <div class="grow relative">
      <LazyCodeEditor ref="editorRef" v-if="fileName && type" :file-name="fileName" :type="type" />
    </div>
  </div>
</template>

<script setup lang="ts">
const route = useRoute();
const router = useRouter();

const fileName = computed(() => route.query.file as string);
const type = computed(() => route.query.type as string);

const editorRef = ref<any>(null);

const { throwSuccess } = useCustomToast();


async function onSave() {
  if (editorRef.value) {
    await editorRef.value.save();
    throwSuccess("File Saved");
  }
}

defineShortcuts({
  meta_s: onSave,
  escape: router.back
});

definePageMeta({
  pageTransition: {
    name: 'slide-right',
    mode: 'out-in'
  }
});

</script>

<style>
.slide-right-enter-active,
.slide-right-leave-active {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.slide-right-enter-from,
.slide-right-leave-to {
  transform: translateX(100%);
}
</style>
