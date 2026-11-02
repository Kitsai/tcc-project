<template>
  <div class="flex flex-col gap-6 p-6 max-w-xl overflow-y-auto">
    <h1 class="text-lg font-semibold">Settings</h1>

    <UForm :state="state" class="flex flex-col gap-y-6" @submit="onSubmit">
      <UFormField label="Author name:" name="authorName"
        help="Used as the author of exported MOJ packages. Leave empty to use the default.">
        <UInput v-model="state.authorName" placeholder="Ballooner" :disabled="!loaded" class="w-full" />
      </UFormField>

      <UFormField label="Max concurrency:" name="maxConcurrency"
        help="How many tests run in parallel. 0 means automatic.">
        <UInputNumber v-model="state.maxConcurrency" :min="0" :max="255" :disabled="!loaded" />
      </UFormField>

      <UButton type="submit" label="Save" :loading="saving" :disabled="!loaded" class="w-fit text-lg px-4" />
    </UForm>
  </div>
</template>

<script setup lang="ts">
import type { Settings } from "~/types/settings";

const settingsStore = useSettingsStore();
const { throwError, throwSuccess } = useCustomToast();

const loaded = ref(false);
const saving = ref(false);

const state = reactive<Settings>({
  maxConcurrency: 0,
  authorName: "",
});

function fillFromStore() {
  const settings = settingsStore.settings;
  if (!settings) return;

  state.maxConcurrency = settings.maxConcurrency;
  state.authorName = settings.authorName;
}

onMounted(async () => {
  await settingsStore.load();

  if (settingsStore.error) {
    throwError(settingsStore.error);
    return;
  }

  fillFromStore();
  loaded.value = true;
});

async function onSubmit() {
  saving.value = true;

  // `save_settings` replaces every field, so both always go together.
  await settingsStore.save({
    maxConcurrency: state.maxConcurrency ?? 0,
    authorName: state.authorName,
  });

  // The store swallows backend errors into `error` instead of throwing.
  if (settingsStore.error) {
    throwError(settingsStore.error);
  } else {
    // The backend trims the name and replaces an empty one with the default.
    fillFromStore();
    throwSuccess("Settings saved.");
  }

  saving.value = false;
}
</script>
