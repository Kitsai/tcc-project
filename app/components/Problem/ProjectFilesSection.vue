<template>
  <div>
    <div class="flex items-center gap-1 px-4 py-2">
      <span class="text-sm font-bold text-highlighted tracking-wide">Project Files</span>
      <div class="ml-auto flex items-center gap-2">
        <ButtonTextField label="Add File" :loading="loading" @submit="onAddFile" />
        <UTooltip text="Refresh">
          <UButton icon="i-lucide-refresh-cw" variant="ghost" size="sm" :loading="loading" @click="getFiles" />
        </UTooltip>
      </div>
    </div>

    <UTable
      v-model:row-selection="rowSelection"
      :loading="loading"
      :data="files"
      :columns="columns"
      :table-options="{ enableMultiRowSelection: false }"
      :ui="{ thead: 'hidden', tr: 'border-l-4 border-transparent transition-all cursor-pointer' }"
      :meta="{
        class: {
          tr: (row: any) => rowSelection[row.id] ? '!bg-primary-50 dark:!bg-primary-950/50 !border-l-primary-500' : ''
        }
      }"
      @select="onSelect"
    >
      <template #fileName-cell="{ row }">
        <div class="flex items-center gap-2">
          <UIcon v-if="rowSelection[row.id]" name="i-lucide-check-circle-2" class="size-4 text-primary-500" />
          <span :class="{ 'font-bold text-primary-600 dark:text-primary-400': rowSelection[row.id] }">
            {{ row.original }}
          </span>
        </div>
      </template>

      <template #actions-cell="{ row }">
        <div class="flex justify-end">
          <UTooltip text="Edit the file">
            <UButton icon="i-lucide-square-pen" color="neutral" variant="ghost" @click.stop="onEdit(row.original)" />
          </UTooltip>
          <UTooltip text="Delete the file">
            <UButton icon="i-lucide-trash" color="error" variant="ghost" @click.stop="onRemove(row.original)" />
          </UTooltip>
        </div>
      </template>
    </UTable>
  </div>
</template>

<script setup lang="ts">
import type { TableColumn, TableRow } from '@nuxt/ui'

const props = defineProps<{
  type: 'validator' | 'checker'
  activeValue?: string | null
}>()

const emit = defineEmits<{
  change: [value: string | undefined]
}>()

const files = ref<string[]>([])
const loading = ref(false)
const rowSelection = ref<Record<string, boolean>>({})

const columns: TableColumn<string>[] = [
  { id: 'fileName', header: 'File Name', accessorFn: (row: string) => row },
  { id: 'actions', header: '' }
]

const { invoke } = useTauri()
const { throwError } = useCustomToast()
const { openInEditor } = useProblemEditor()

watch(
  [() => props.activeValue, files],
  ([val, fileList]) => {
    if (!val || val.startsWith('@default:')) {
      rowSelection.value = {}
      return
    }
    const idx = (fileList as string[]).indexOf(val)
    rowSelection.value = idx >= 0 ? { [idx.toString()]: true } : {}
  },
  { immediate: true }
)

async function getFiles() {
  loading.value = true
  try {
    files.value = await invoke<string[]>('get_files')
  } catch (e) {
    console.error('[ProjectFilesSection] Error in getFiles:', e)
  }
  loading.value = false
}

async function onSelect(_: Event, row: TableRow<string>) {
  const previous = props.activeValue
  emit('change', row.original)
  try {
    await invoke('select_problem_file', { fileType: props.type, file: row.original })
  } catch (e) {
    emit('change', previous ?? undefined)
    throwError('Failed to select file: ' + e)
  }
}

async function onAddFile(name: string) {
  try {
    await invoke('create_file_on_dir', { dir: 'files', fileName: name })
    await getFiles()
  } catch (e) {
    throwError('Failed to create file')
    console.error(e)
  }
}

function onEdit(filename: string) {
  openInEditor(props.type, filename)
}

async function onRemove(filename: string) {
  try {
    await invoke('delete_file_on_dir', { dir: 'files', fileName: filename })
    await getFiles()
  } catch (e) {
    throwError('Failed to remove file')
    console.error(e)
  }
}

onMounted(async () => {
  await getFiles()
})
</script>
