<template>
  <UModal class="p-10" v-model:open="open">
    <template #content>
      <UForm class="flex flex-col gap-4" @submit="onSubmit">
        <UFormField label="Test Number" name="id">
          <UInput type="number" v-model="state.id" />
        </UFormField>
        <UTooltip v-if="!copyFrom" text='Use "===" to separate inputs, one verdict per line'>
          <UFormField label="Multiple Tests?" name="mult">
            <UCheckbox v-model="state.mult" />
          </UFormField>
        </UTooltip>
        <UFormField label="Input(s)" name="input">
          <UTextarea v-model="state.input" />
        </UFormField>
        <UFormField label="Verdict(s)" name="verdict">
          <UTextarea v-if="state.mult" v-model="state.verdict" />
          <ProblemValidatorResultSelect v-else v-model="state.verdict" />
        </UFormField>
        <UButton class="w-fit" type="submit" label="Create" />
      </UForm>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { ValidatorTest, ValidatorTestCreateDto } from '~/types/validator/ValidatorTest';

const { invoke } = useTauri();
const { throwError } = useCustomToast();

const open = defineModel<boolean>('open', { required: true });

const props = defineProps<{
  copyFrom?: ValidatorTest | null
}>();

const emit = defineEmits<{
  success: [],
}>();

const state = reactive<ValidatorTestCreateDto>({
  id: 0,
  mult: false,
  input: "",
  verdict: "VALID"
});

watch(open, async (val) => {
  if (val) {
    state.id = await invoke<number>("get_next_validator_test_id");
    if (props.copyFrom) {
      state.input = props.copyFrom.input;
      state.verdict = props.copyFrom.expected;
      state.mult = false;
    }
  } else {
    Object.assign(state, { mult: false, input: "", verdict: "VALID" });
  }
});

function close() {
  open.value = false;
}

async function onSubmit() {

  try {
    await invoke("create_validator_test", { test: { ...state } })
    emit("success");
    close();
  } catch (e) {
    throwError("Failed to create: " + e);
    console.error(e);
  }
}

</script>
