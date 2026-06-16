<template>
  <div class="flex justify-end py-2 gap-2">
    <ProblemSolutionNewFileButton />
    <ProblemSolutionAddFilesButton />
  </div>
  <UTable :columns="columns" :data="data">
    <template #tag-cell="{ row }">
      <span :class="success_tag(row.original.tag) ? 'text-success' : 'text-error'">{{ tag_to_text(row.original.tag)
        }}</span>
    </template>

    <template #actions-cell="{ row }">
    </template>

  </UTable>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import { success_tag, tag_to_text, type SolutionDescription } from '~/types/solution';

const columns: TableColumn<SolutionDescription>[] = [
  {
    id: "author",
    header: "Author",
    accessorKey: "author"
  },
  {
    id: "file_name",
    header: "Name",
    accessorKey: "file_name",
  },
  {
    id: "modified",
    header: "Modified",
    accessorKey: "change_time",
  },
  {
    id: "tag",
    header: "Type"
  },
  {
    id: "actions",
  }
]

const data: SolutionDescription[] = [
  {
    author: "Kitsai",
    file_name: "solution.cpp",
    tag: 'MAIN',
    change_time: "2026-06-16 19:38",
  },
  {
    author: "Kitsai",
    file_name: "wrong.cpp",
    tag: 'WRONG_ANSWER',
    change_time: "2026-06-16 19:48",
  }
]
</script>
