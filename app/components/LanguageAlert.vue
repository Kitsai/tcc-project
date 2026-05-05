<template>
  <LazyUAlert title="Language not available!" color="error" v-if="lang.ready && !lang.allAvailable">
    <template #description>
      <div class="flex flex-col gap-1">
        <span v-for="msg in messages" :key="msg">{{ msg }}</span>
      </div>
    </template>
  </LazyUAlert>
</template>

<script setup lang="ts">
const lang = useLanguageDetails();

const messages = computed(() => {
  const ret: string[] = [];

  if (!lang.config.gpp) ret.push("g++ is not available");
  if (!lang.config.python3) ret.push("python3 is not available");

  return ret;
});
</script>
