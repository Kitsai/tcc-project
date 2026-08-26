<template>
    <UModal v-model:open="open">
       <template #content>
            <div class="p-10 overflow-y-auto max-h-[80vh]">
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
                  <template v-else>
                    <UFormField label="Generator" name="generatorFile">
                      <USelect v-model="state.generatorFile" :items="generatorFiles" class="w-full"
                        placeholder="Select a generator file" />
                    </UFormField>
                    <UFormField label="Args" name="args">
                      <UInput type="text" v-model="state.args" class="w-full" placeholder="1 100" />
                    </UFormField>
                  </template>
                  <UFormField label="Use in statements" name="example">
                      <UCheckbox v-model="state.example"/>
                  </UFormField>
                  <UFormField label="Description" name="description">
                      <UTextarea class="w-full" v-model="state.description"/>
                  </UFormField>

                  <UButton class="w-fit" type="submit" label="Create" />
              </UForm>
            </div>
       </template>
    </UModal>
</template>

<script setup lang="ts">
import type { TestDefinitionCreateDto, TestType } from '~/types/tests/definition';

const { invoke } = useTauri();
const { throwError } = useCustomToast();
const generatorFiles = useGeneratorFiles();

const open = defineModel<boolean>('open', { required: true });

const emit = defineEmits<{
  success: []
}>();

const state = reactive({
  id: 0,
  testType: 'Manual' as TestType,
  content: '',
  generatorFile: '',
  args: '',
  example: false,
  description: ''
})

watch(open, async (val) => {
  if (val) {
    state.id = await invoke<number>("get_next_test_id");
  } else {
    Object.assign(state, { id: 0, testType: 'Manual', content: '', generatorFile: '', args: '', example: false, description: '' });
  }
});

async function OnSubmit() {
  const content = state.testType === 'Script'
    ? [state.generatorFile, state.args].filter(Boolean).join(' ').trim()
    : state.content;

  const test: TestDefinitionCreateDto = {
    id: state.id,
    testType: state.testType,
    content,
    example: state.example,
    description: state.description,
  };

  try {
    await invoke("create_test", { test });
    emit("success");
    open.value = false;
  } catch (e) {
    console.error(e);
    throwError("Failed to create: " + e);
  }
}
</script>
