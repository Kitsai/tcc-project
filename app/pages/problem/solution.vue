<template>
  <div class="flex justify-end py-2 gap-2">
    <ProblemSolutionNewFileButton />
    <ProblemSolutionAddFilesButton />
  </div>
  <UTable :columns="columns" :data="data">
    <template #tag-cell="{ row }">
      <USelect :model-value="row.original.tag" :items="tagItems" variant="ghost"
        :class="success_tag(row.original.tag) ? 'text-success' : 'text-error'"
        @update:model-value="(tag) => onTagChange(row.index, tag)" />
    </template>

    <template #actions-cell="{ row }">
      <div class="flex items-center gap-2">
        <UTooltip text="Delete this solution">
          <UButton icon="i-lucide-trash" color="error" variant="ghost" @click.stop="onDeleteSolution(row.original)" />
        </UTooltip>
        <UTooltip text="Edit this solution">
          <UButton icon="i-lucide-square-pen" color="neutral" variant="ghost"
            @click.stop="onEditSolution(row.original)" />
        </UTooltip>
      </div>
    </template>

  </UTable>
</template>

<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui';
import { SOLUTION_TAGS, success_tag, tag_to_text, type SolutionDescription, type SolutionTag } from '~/types/solution/SolutionDescription';

const { invoke } = useTauri();
const { throwError } = useCustomToast()

const tagItems = SOLUTION_TAGS.map((tag) => ({ label: tag_to_text(tag), value: tag }));

const data = ref<SolutionDescription[]>([]);

async function updateData() {
  try {
    data.value = await invoke<SolutionDescription[]>("get_solutions");
  } catch (e) {
    console.error(e);
    throwError("Failed to get solutions");
  }
}

function onTagChange(index: number, tag: SolutionTag) {
  if (data.value[index])
    data.value[index].tag = tag;
}

async function onDeleteSolution(solution: SolutionDescription) {
  try {
    await invoke("delete_solution");
  } catch (e) {
    console.error(e);
    throwError("Failed to delete solution");
  }

  updateData();
}

function onEditSolution(solution: SolutionDescription) {

}

onMounted(updateData);

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

</script>
