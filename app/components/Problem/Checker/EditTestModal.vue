<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-10 overflow-y-auto max-h-[80vh] flex flex-col gap-3">
        <h1>Edit test {{ props.test?.id }}</h1>
        <UForm class="flex flex-col gap-4" @submit="onSubmit">
          <UFormField label="Input" name="input" class="w-full">
            <UTextarea class="w-full" v-model="state.input" />
          </UFormField>
          <UFormField label="Output" name="output" class="w-full">
            <UTextarea class="w-full" v-model="state.output" />
          </UFormField>
          <UFormField label="Answer" name="answer" class="w-full">
            <UTextarea class="w-full" v-model="state.answer" />
          </UFormField>
          <UFormField label="Verdict" name="verdict" class="w-full">
            <ProblemCheckerVerdictSelect v-model="state.verdict" />
          </UFormField>
          <UButton type="submit" label="Save" class="w-fit" />
        </UForm>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { CheckerTest, CheckerTestEditDto } from '~/types/checker/CheckerTest';

const { invoke } = useTauri();
const { throwError } = useCustomToast();

const open = defineModel<boolean>('open', { required: true });

const props = defineProps<{
  test: CheckerTest | null
}>();

const emit = defineEmits<{
  success: []
}>();

const state = reactive({
  input: "",
  output: "",
  answer: "",
  verdict: "",
});

watch(open, (val) => {
  if (val && props.test) {
    state.input = props.test.input;
    state.output = props.test.output;
    state.answer = props.test.answer;
    state.verdict = props.test.expected;
  }
});

async function onSubmit() {
  if (!props.test) return;

  const dto: CheckerTestEditDto = {
    id: props.test.id,
    input: state.input,
    output: state.output,
    answer: state.answer,
    verdict: state.verdict,
  };
  try {
    await invoke("edit_checker_test", { dto: { ...dto } });
    emit("success");
    open.value = false;
  } catch (e) {
    throwError("Failed to edit test: " + e);
    console.error(e);
  }
}
</script>
