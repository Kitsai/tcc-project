<template>
    <UModal v-model:open="isOpen">
        <UButton class="w-fit text-lg" label="Criar Problema" />

        <template #content>
            <UHeader title="Crie um novo problema" to="" />
            <UForm
                @submit="onSubmit"
                class="flex flex-col gap-4 justify-center items-center py-20"
            >
                <UFormField label="Nome do Problema">
                    <UInput type="text" v-model="problemName" />
                </UFormField>
                <UFormField label="Local do Problema">
                    <UButton
                        label="Local do Projeto"
                        color="secondary"
                        type="button"
                        v-if="
                            problemFolder === null || problemFolder.length === 0
                        "
                        @click="onSelectPath"
                    />
                    <LazyUInput disabled :value="problemFolder" v-else />
                </UFormField>

                <UButton
                    class="w-fit text-lg px-5"
                    type="submit"
                    :disabled="
                        problemName.length === 0 ||
                        problemFolder === null ||
                        problemFolder.length === 0
                    "
                >
                    Criar
                </UButton>
            </UForm>
        </template>
    </UModal>
</template>

<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";

const isOpen = ref(false);

const problemName = ref("");
const problemFolder = ref<string | null>(null);

async function onSelectPath() {
    problemFolder.value = await open({
        multiple: false,
        directory: true,
    });
}

async function onSubmit() {
    const { invoke } = useTauri();

    try {
        invoke("create_problem", {
            name: problemName.value,
            path: problemFolder.value,
        });

        close();
    } catch {
        console.log("err");
    }
}

function close() {
    problemFolder.value = null;
    problemName.value = "";
    isOpen.value = false;
}
</script>
