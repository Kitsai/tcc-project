<template>
  <UModal v-model:open="isOpen">
    <UButton class="w-fit text-lg" label="Importar do Polygon" color="secondary" />

    <template #content>
      <div class="overflow-y-auto max-h-[80vh]">
        <UHeader title="Importar pacote do Polygon" to="" :toggle="false" />

        <div v-if="warnings.length > 0" class="flex flex-col gap-4 p-6">
          <UAlert
            v-for="(warning, i) in warnings"
            :key="i"
            color="warning"
            variant="subtle"
            :description="warning"
          />
          <UButton label="Continuar" class="self-end" @click="goToProblem" />
        </div>

        <UForm v-else @submit="onSubmit" class="flex flex-col gap-4 justify-center items-center py-20">
          <UFormField label="Pasta do Pacote Polygon">
            <UButton v-if="!sourceFolder" label="Selecionar Pasta" color="secondary" type="button" @click="onSelectSource" />
            <LazyUInput v-else :value="sourceFolder" @click="onSelectSource" />
          </UFormField>

          <UFormField label="Nome do Problema">
            <UInput type="text" v-model="problemName" />
          </UFormField>

          <UFormField label="Local de Destino">
            <UButton v-if="!destFolder" label="Local do Projeto" color="secondary" type="button" @click="onSelectDest" />
            <LazyUInput v-else :value="destFolder" @click="onSelectDest" />
          </UFormField>

          <UButton
            class="w-fit text-lg px-5"
            type="submit"
            :disabled="!sourceFolder || !destFolder || problemName.length === 0"
            :loading="problems.loading"
          >
            Importar
          </UButton>
        </UForm>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";

const isOpen = ref(false);
const sourceFolder = ref<string | null>(null);
const destFolder = ref<string | null>(null);
const problemName = ref("");

const problems = useProblems();
const warnings = computed(() => problems.importWarnings);

async function onSelectSource() {
  const picked = await open({ multiple: false, directory: true });
  if (!picked) return;

  sourceFolder.value = picked;
  if (!problemName.value) {
    problemName.value = picked.split(/[\\/]/).filter(Boolean).pop() ?? "";
  }
}

async function onSelectDest() {
  destFolder.value = await open({ multiple: false, directory: true });
}

async function onSubmit() {
  await problems.importPolygon(sourceFolder.value!, destFolder.value!, problemName.value);

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