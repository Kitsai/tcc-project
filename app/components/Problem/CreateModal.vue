<template>
    <UModal>
        <UButton class="w-fit text-lg" label="Criar Problema" />

        <template #content>
            <UForm
                @submit="onSubmit"
                class="flex flex-col gap-3 justify-center items-center"
            >
                <UFormField label="Nome do Problema">
                    <UInput type="text" v-model="problemName" />
                </UFormField>
                <UFormField label="Local do Problema">
                    <UInput type="file" />
                </UFormField>

                <UButton class="w-fit" type="submit"> Criar </UButton>
            </UForm>
        </template>
    </UModal>
</template>

<script setup lang="ts">
const problemName = ref("");

function onSubmit() {}
</script>
