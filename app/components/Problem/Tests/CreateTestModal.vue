<template>
    <UModal class="p-10" v-model:open="open">
       <template #content>
            <UForm class="flex flex-col gap-4" @submit="OnSubmit">
                <UFormField label="Test Number" name="id">
                  <UInput type="number" v-model="state.id"/>
                </UFormField>
                <UFormField label="Type" name="type">
                  <ProblemTestsSelectField v-model="state.testType"/>
                </UFormField>
                <UFormField v-if="state.testType === 'Manual'" label="Data" name="content">
                  <UTextarea class="w-full" v-model="state.content"/>
                </UFormField>
                <UFormField v-else label="Script Line" name="content">
                    <UInput type="text" v-model="state.content" />
                </UFormField>
                <UFormField label="Use in statements" name="example">
                    <UCheckbox v-model="state.example"/>
                </UFormField>
                <UFormField label="Description" name="description">
                    <UTextarea class="w-full" v-model="state.description"/>
                </UFormField>

                <UButton class="w-fit" type="submit" label="Create" />
            </UForm>
       </template>
    </UModal>
</template>

<script setup lang="ts">
import type { TestDefinition } from '~/types/tests/definition';


const open = defineModel<boolean>('open', { required: true });

const state = reactive<TestDefinition>({
  id: 0,
  testType: 'Manual',
  content: '',
  example: false,
  description: ''
})

async function OnSubmit() {
  open.value = false;
}
</script>
