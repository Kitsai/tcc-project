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
import type { TestDefinitionCreateDto } from '~/types/tests/definition';

const { invoke } = useTauri();
const { throwError } = useCustomToast();

const open = defineModel<boolean>('open', { required: true });

const emit = defineEmits<{
  success: []
}>();

const state = reactive<TestDefinitionCreateDto>({
  id: 0,
  testType: 'Manual',
  content: '',
  example: false,
  description: ''
})

watch(open, async (val) => {
  if (val) {
    state.id = await invoke<number>("get_next_test_id");
  } else {
    Object.assign(state, { id: 0, testType: 'Manual', content: '', example: false, description: '' });
  }
});

async function OnSubmit() {
  try {
    await invoke("create_test", { test: { ...state } });
    emit("success");
    open.value = false;
  } catch (e) {
    console.error(e);
    throwError("Failed to create: " + e);
  }
}
</script>
