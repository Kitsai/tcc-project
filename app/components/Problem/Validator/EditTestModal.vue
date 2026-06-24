<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-10 flex flex-col gap-3">
        <h1>Edit test {{ props.test?.id }}</h1>
        <UForm class="flex flex-col gap-4" @submit="onSubmit">
          <UFormField label="Input" name="input">
            <UTextarea v-model="state.input" />
          </UFormField>
          <UFormField label="Verdict" name="verdict">
            <ProblemValidatorResultSelect v-model="state.verdict" />
          </UFormField>
          <UButton type="submit" label="Save" class="w-fit" />
        </UForm>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { ValidatorTest, ValidatorTestEditDto } from '~/types/validator/ValidatorTest';

const { invoke } = useTauri();
const { throwError } = useCustomToast();

const open = defineModel<boolean>('open', { required: true });

const props = defineProps<{
  test: ValidatorTest | null
}>();

const emit = defineEmits<{
  success: []
}>()

const state = reactive({
  input: "",
  verdict: "",
});

watch(open, (val) => {
  if (val && props.test) {
    state.input = props.test.input;
    state.verdict = props.test.expected;
  }
});

async function onSubmit() {
  if (!props.test) return;

  let dto: ValidatorTestEditDto = { id: props.test.id, input: state.input, verdict: state.verdict };
  try {
    await invoke("edit_validator_test", { dto: { ...dto } })
    emit("success");
    open.value = false;
  } catch (e) {
    throwError("Failed to edit test: " + e);
    console.error(e);
  }
}
</script>
