<template>
    <UTabs :items="items"> </UTabs>
</template>

<script setup lang="ts">
const route = useRoute();
const problems = useProblems();

const items = [
    {
        label: "Statement",
        slot: "statement",
    },
    {
        label: "Checker",
        slot: "checkerPage",
    },
    {
        label: "Validator",
        slot: "validatorPage",
    },
    {
        label: "Tests",
        slot: "TestsPage",
    },
    {
        label: "Stresses",
        slot: "StressesPage",
    },
    {
        label: "Solutions",
        slot: "TestsPage",
    },
];
</script>
