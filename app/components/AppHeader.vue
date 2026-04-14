<template>
    <UHeader
        :title="'Placeholder' + problemName"
        to="/"
        toggleSide="left"
        class="bg-slate-200 dark:bg-slate-950"
    >
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
        <template #body>
            <UNavigationMenu
                :items="items"
                orientation="vertical"
                class="-mx-2.5"
            />
        </template>
    </UHeader>
</template>

<script setup lang="ts">
import type { NavigationMenuItem } from "@nuxt/ui";

const items = computed<NavigationMenuItem[]>(() => [
    {
        label: "Test",
        to: "/test",
        active: true,
    },
]);

const problems = useProblems();

const problemName = computed(() =>
    problems.currentProblem === null
        ? ""
        : " - " + problems.currentProblem.name,
);
</script>
