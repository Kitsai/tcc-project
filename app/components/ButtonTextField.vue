<template>
  <div class="flex justify-end items-center min-h-10">
    <Transition name="morph" mode="out-in">
      <div v-if="!textOpen" key="button">
        <UButton type="button" :label="props.label" :loading="props.loading" variant="subtle" @click="onAddClick" />
      </div>
      <div v-else key="form" class="flex items-center gap-2">
        <UInput v-model="state.name" placeholder="Name..." class="w-48" @keyup.enter="onSubmit"
          @keydown.esc="onCancel" />
        <div class="flex items-center gap-1">
          <UButton icon="i-lucide-check" color="primary" variant="ghost" @click="onSubmit" />
          <UButton icon="i-lucide-x" color="neutral" variant="ghost" @click="onCancel" />
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  label?: string;
  loading?: boolean;
}>();

const emit = defineEmits<{
  submit: [name: string];
  cancel: [];
}>();

const textOpen = ref(false);

const state = reactive({
  name: "",
});

function onAddClick() {
  textOpen.value = true;
}

function onSubmit() {
  if (state.name.trim()) {
    emit("submit", state.name);
    state.name = "";
    textOpen.value = false;
  }
}

function onCancel() {
  textOpen.value = false;
  state.name = "";
  emit("cancel");
}
</script>

<style scoped>
.morph-enter-active,
.morph-leave-active {
  transition: all 0.2s ease;
}

.morph-enter-from,
.morph-leave-to {
  opacity: 0;
  transform: translateY(4px);
}
</style>
