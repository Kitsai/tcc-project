<template>
    <UApp>
        <UHeader
            title="Placeholder"
            to="/"
            :toggle="false"
            class="bg-slate-200 dark:bg-slate-950"
        >
            <template #right>
                <UColorModeButton />
            </template>
        </UHeader>

        <USeparator color="secondary" />

        <UMain>
            <NuxtLayout>
                <NuxtPage />
            </NuxtLayout>
        </UMain>
    </UApp>
</template>
