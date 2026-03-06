<template>
  <label :for="props.id"></label>
  <select :name="props.name" :id="props.id">
    <option v-for="entry in options" :key="entry.text" :value="entry.value">
      {{ entry.text }}
    </option>
  </select>
</template>

<script setup lang="ts">
import type { Entry } from '~/types/common/entry';

interface Props {
  id: string,
  name: string,
  options: Entry[]
}

const props = defineProps<Props>();
</script>
