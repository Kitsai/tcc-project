<template>
    <UHeader to="/" :toggle="false" class="bg-slate-200 dark:bg-slate-950">
        <template #title>
            <div class="font-outfit">
                <span class="text-2xl">
                    <span class="text-primary-600 dark:text-primary-400"
                        >Balloon</span
                    >
                    <span class="">er</span>
                </span>
                <span>{{ problemName }}</span>
            </div>
        </template>
        <template #right>
            <UColorModeButton />
            <UTooltip text="Abrir configurações" :kbds="['meta', 'Escape']">
                <UButton
                    color="neutral"
                    variant="ghost"
                    to="/settings"
                    icon="i-lucide-settings"
                />
            </UTooltip>
        </template>
    </UHeader>
</template>

<script setup lang="ts">
const problemStore = useProblems();
const { isProblemOpened, currentName } = storeToRefs(problemStore);

const problemName = computed(() =>
    isProblemOpened.value ? " - " + currentName.value : "",
);
</script>
