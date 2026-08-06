<template>
    <USelect v-model="model"
        :items="['Script', 'Manual']"
    />
</template>

<script setup lang="ts">
const model = defineModel<string>({required: true})
</script>
