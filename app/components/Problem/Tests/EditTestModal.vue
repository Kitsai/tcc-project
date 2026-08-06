<template>
  <UModal class="p-10" v-model:open="open">
    <template #content>
      <div class="p-10 flex flex-col gap-3">
        <h1>Edit test {{ props.test?.id }}</h1>
        <UForm class="flex flex-col gap-4" @submit="OnSubmit">
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

          <UButton class="w-fit" type="submit" label="Save" />
        </UForm>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { TestDefinition } from '~/types/tests/definition';

const open = defineModel<boolean>('open', { required: true });

const props = defineProps<{
  test: TestDefinition | null
}>();

const emit = defineEmits<{
  success: [test: TestDefinition]
}>();

const state = reactive({
  testType: 'Manual' as TestDefinition['testType'],
  content: '',
  example: false,
  description: ''
});

watch(open, (val) => {
  if (val && props.test) {
    state.testType = props.test.testType;
    state.content = props.test.content;
    state.example = props.test.example;
    state.description = props.test.description;
  }
});

async function OnSubmit() {
  if (!props.test) return;

  emit('success', {
    id: props.test.id,
    testType: state.testType,
    content: state.content,
    example: state.example,
    description: state.description
  });
  open.value = false;
}
</script>
