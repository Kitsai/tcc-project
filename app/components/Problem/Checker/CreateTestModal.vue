<template>
  <UModal v-model:open="open">
    <template #content>
      <div class="p-10 overflow-y-auto max-h-[80vh]">
        <UForm class="flex flex-col gap-4" @submit="onSubmit">
          <UFormField label="Test Number" name="id">
            <UInput type="number" v-model="state.id" />
          </UFormField>
          <UTooltip v-if="!copyFrom" text='Use "===" to separate inputs, one verdict per line'>
            <UFormField label="Multiple Tests?" name="mult">
              <UCheckbox v-model="state.mult" />
            </UFormField>
          </UTooltip>
          <UFormField label="Input(s)" name="input" class="w-full">
            <UTextarea class="w-full" v-model="state.input" />
          </UFormField>
          <UFormField label="Output(s)" name="output" class="w-full">
            <UTextarea class="w-full" v-model="state.output" />
          </UFormField>
          <UFormField label="Answer(s)" name="answer" class="w-full">
            <UTextarea class="w-full" v-model="state.answer" />
          </UFormField>
          <UFormField label="Verdict(s)" name="verdict" class="w-full">
            <UTextarea v-if="state.mult" class="w-full" v-model="state.verdict" />
            <ProblemCheckerVerdictSelect v-else v-model="state.verdict" />
          </UFormField>
          <UButton class="w-fit" type="submit" label="Create" />
        </UForm>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { CheckerTest, CheckerTestCreateDto } from '~/types/checker/CheckerTest';

const { invoke } = useTauri();
const { throwError } = useCustomToast();

const open = defineModel<boolean>('open', { required: true });

const props = defineProps<{
  copyFrom?: CheckerTest | null
}>();

const emit = defineEmits<{
  success: [],
}>();

const state = reactive<CheckerTestCreateDto>({
  id: 0,
  mult: false,
  input: "",
  output: "",
  answer: "",
  verdict: "OK"
});

watch(open, async (val) => {
  if (val) {
    state.id = await invoke<number>("get_next_checker_test_id");
    if (props.copyFrom) {
      state.input = props.copyFrom.input;
      state.output = props.copyFrom.output;
      state.answer = props.copyFrom.answer;
      state.verdict = props.copyFrom.expected;
      state.mult = false;
    }
  } else {
    Object.assign(state, { mult: false, input: "", output: "", answer: "", verdict: "OK" });
  }
});

function close() {
  open.value = false;
}

async function onSubmit() {
  try {
    await invoke("create_checker_test", { test: { ...state } });
    emit("success");
    close();
  } catch (e) {
    throwError("Failed to create: " + e);
    console.error(e);
  }
}
</script>
