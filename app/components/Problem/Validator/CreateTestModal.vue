<template>
  <UModal class="p-10" v-model:open="open">
    <template #content>
      <UForm class="flex flex-col gap-4" @submit="onSubmit">
        <UFormField label="Test Number" name="id">
          <UInput type="number" v-model="state.id" />
        </UFormField>
        <UTooltip text='Use "===" to separate inputs, one verdicts per line'>
          <UFormField label="Multiple Tests?" name="mult">
            <UCheckbox v-model="state.mult" />
          </UFormField>
        </UTooltip>
        <UFormField label="Input(s)" name="input">
          <UTextarea v-model="state.input" />
        </UFormField>
        <UFormField label="Verdict(s)" name="verdict">
          <UTextarea v-model="state.verdict" />
        </UFormField>
        <UButton class="w-fit" type="submit" label="Create" />
      </UForm>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { FormSubmitEvent } from '@nuxt/ui';
const { invoke } = useTauri();
const { throwError } = useCustomToast();

const open = defineModel<boolean>('open', { required: true });

const emit = defineEmits<{
  success: [],
}>();

const state = reactive<ValidatorTestCreateDto>({
  id: 0,
  mult: false,
  input: "",
  verdict: ""
});

type Schema = typeof state;

watch(open, async (val) => {
  if (val) {
    state.id = await invoke<number>("get_next_validator_test_id");
  } else {
    Object.assign(state, { mult: false, input: "", verdict: "" });
  }
});

function close() {
  open.value = false;
}

async function onSubmit(_event: FormSubmitEvent<Schema>) {

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
