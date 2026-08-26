<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-10 overflow-y-auto max-h-[80vh] flex flex-col gap-3">
        <h1>Edit test {{ props.test?.id }}</h1>
        <UForm class="flex flex-col gap-4" @submit="OnSubmit">
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

          <UButton class="w-fit" type="submit" label="Save" />
        </UForm>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { TestDefinition, TestDefinitionEditDto, TestType } from '~/types/tests/definition';

const { invoke } = useTauri();
const { throwError } = useCustomToast();
const generatorFiles = useGeneratorFiles();

const open = defineModel<boolean>('open', { required: true });

const props = defineProps<{
  test: TestDefinition | null
}>();

const emit = defineEmits<{
  success: [test: TestDefinition]
}>();

const state = reactive({
  testType: 'Manual' as TestType,
  content: '',
  generatorFile: '',
  args: '',
  example: false,
  description: ''
});

watch(open, (val) => {
  if (val && props.test) {
    state.testType = props.test.testType;
    state.content = props.test.content;
    state.example = props.test.example;
    state.description = props.test.description;

    if (props.test.testType === 'Script') {
      const [first, ...rest] = props.test.content.trim().split(/\s+/);
      state.generatorFile = first ?? '';
      state.args = rest.join(' ');
    } else {
      state.generatorFile = '';
      state.args = '';
    }
  }
});

async function OnSubmit() {
  if (!props.test) return;

  const content = state.testType === 'Script'
    ? [state.generatorFile, state.args].filter(Boolean).join(' ').trim()
    : state.content;

  const dto: TestDefinitionEditDto = {
    id: props.test.id,
    testType: state.testType,
    content,
    example: state.example,
    description: state.description
  };

  try {
    const updated = await invoke<TestDefinition>("edit_test", { dto: { ...dto } });
    emit('success', updated);
    open.value = false;
  } catch (e) {
    console.error(e);
    throwError("Failed to edit test: " + e);
  }
}
</script>
