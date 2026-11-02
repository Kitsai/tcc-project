<template>
  <UModal v-model:open="isOpen">
    <UButton class="w-fit text-lg" label="Import Problem" color="secondary" />

    <template #content>
      <div class="overflow-y-auto max-h-[80vh]">
        <UHeader title="Import Problem" to="" :toggle="false" />

        <div v-if="warnings.length > 0" class="flex flex-col gap-4 p-6">
          <UAlert v-for="(warning, i) in warnings" :key="i" color="warning" variant="subtle" :description="warning" />
          <UButton label="Continue" class="self-end" @click="goToProblem" />
        </div>

        <UForm v-else @submit="onSubmit" class="flex flex-col items-center py-20">
          <div class="flex flex-col gap-4 items-start">
            <UFormField label="Format">
              <USelect v-model="format" :items="formatItems" class="w-56" />
            </UFormField>

            <UFormField :label="format === 'moj' ? 'MOJ package (.tar.gz or folder)' : 'Polygon package folder'">
              <div v-if="!source" class="flex gap-2">
                <UButton v-if="format === 'moj'" label="Select File" color="secondary" type="button"
                  @click="onSelectSource(false)" />
                <UButton label="Select Folder" color="secondary"
                  type="button" @click="onSelectSource(true)" />
              </div>
              <LazyUInput v-else :value="source" @click="onSelectSource(format === 'polygon' || sourceIsFolder)" />
            </UFormField>

            <UFormField label="Problem Name">
              <UInput type="text" v-model="problemName" />
            </UFormField>

            <UFormField label="Destination">
              <UButton v-if="!destFolder" label="Project Location" color="secondary" type="button"
                @click="onSelectDest" />
              <LazyUInput v-else :value="destFolder" @click="onSelectDest" />
            </UFormField>

            <UButton class="w-fit text-lg px-5 mt-6" type="submit"
              :disabled="!source || !destFolder || problemName.length === 0" :loading="problems.loading">
              Import
            </UButton>
          </div>
        </UForm>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";

import type { ImportFormat } from "~/types/problem/import_format";

const formatItems = [
  { label: "MOJ (.tar.gz or folder)", value: "moj" },
  { label: "Polygon (folder)", value: "polygon" },
];

const isOpen = ref(false);
const format = ref<ImportFormat>("moj");
const source = ref<string | null>(null);
const sourceIsFolder = ref(true);
const destFolder = ref<string | null>(null);
const problemName = ref("");

const problems = useProblems();
const warnings = computed(() => problems.importWarnings);

// A MOJ package can be an archive or an unpacked folder, so the picker is
// told which one to offer; Polygon packages are always folders.
async function onSelectSource(directory: boolean) {
  const picked = await open({
    multiple: false,
    directory,
    filters: directory ? undefined : [{ name: "MOJ package", extensions: ["gz", "tgz", "tar"] }],
  });
  if (!picked) return;

  source.value = picked;
  sourceIsFolder.value = directory;
  if (!problemName.value) {
    const base = picked.split(/[\\/]/).filter(Boolean).pop() ?? "";
    problemName.value = base.replace(/\.(tar\.gz|tgz|tar)$/i, "");
  }
}

// The two formats take different kinds of source, so a pick made for one
// must not carry over to the other.
watch(format, () => {
  source.value = null;
});

async function onSelectDest() {
  destFolder.value = await open({ multiple: false, directory: true });
}

async function onSubmit() {
  await problems.importPackage(format.value, source.value!, destFolder.value!, problemName.value);

  if (problems.error !== null) {
    const { throwError } = useCustomToast();
    throwError(problems.error);
    return;
  }

  if (problems.importWarnings.length === 0) {
    goToProblem();
  }
}

function goToProblem() {
  navigateTo({
    path: "/problem",
    query: {
      path: destFolder.value! + "/" + problemName.value!,
    },
  });
}
</script>
