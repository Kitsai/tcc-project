<template>
  <div>
    <UTable :columns="columns">
    </UTable>
  </div>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import type { TestDefinition } from '~/types/tests/definition';


const columns: TableColumn<TestDefinition>[] = [
  {
    id: "id",
    header: "#",
    accessorKey: "id"
  },
  {
    id: 'content',
    header: 'Content',
  },
  {
    id: 'description',
    header: 'Desc',
    accessorKey: 'description'
  },
  {
    id: 'example',
    header: 'example',
  },
  {
    id: 'actions',
    header: ''
  }
]
</script>
