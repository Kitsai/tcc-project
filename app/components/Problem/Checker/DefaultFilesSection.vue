<template>
  <div>
    <div class="flex items-center gap-1 px-4 py-2 cursor-pointer select-none hover:bg-elevated/50 transition-colors"
      @click="open = !open">
      <UIcon :name="open ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'" class="size-4 text-dimmed" />
      <span class="text-sm font-bold text-highlighted tracking-wide">Default Checkers</span>
    </div>

    <div v-if="open">
      <UTable :data="files" :columns="columns" v-model:row-selection="rowSelection"
        :table-options="{ enableMultiRowSelection: false }"
        :ui="{ thead: 'hidden', tr: 'border-l-4 border-transparent transition-all cursor-pointer' }" :meta="{
          class: {
            tr: (row: any) => rowSelection[row.id] ? '!bg-primary-50 dark:!bg-primary-950/50 !border-l-primary-500' : ''
          }
        }" @select="onSelect">
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
            <UTooltip text="View source">
              <UButton icon="i-lucide-eye" color="neutral" variant="ghost" @click.stop="onView(row.original)" />
            </UTooltip>
          </div>
        </template>
      </UTable>
    </div>

    <ProblemCheckerViewDefaultFileModal v-model:open="viewOpen" :file-name="viewingFile" />
  </div>
</template>

<script setup lang="ts">
import type { TableColumn, TableRow } from '@nuxt/ui'

const props = defineProps<{
  activeValue?: string | null
}>()

const emit = defineEmits<{
  change: [value: string | undefined]
}>()

const open = ref(true)
const files = ref<string[]>([])
const rowSelection = ref<Record<string, boolean>>({})

const columns: TableColumn<string>[] = [
  { id: 'fileName', header: 'File Name', accessorFn: (row: string) => row },
  { id: 'actions', header: '' }
]

const { invoke } = useTauri()
const { throwError } = useCustomToast()

const viewOpen = ref(false)
const viewingFile = ref<string | null>(null)

function onView(name: string) {
  viewingFile.value = name
  viewOpen.value = true
}

watch(
  [() => props.activeValue, files],
  ([val, fileList]) => {
    if (!val?.startsWith('@default:')) {
      rowSelection.value = {}
      return
    }
    const name = val.slice('@default:'.length)
    const idx = (fileList as string[]).indexOf(name)
    rowSelection.value = idx >= 0 ? { [idx.toString()]: true } : {}
  },
  { immediate: true }
)

async function onSelect(_: Event, row: TableRow<string>) {
  const previous = props.activeValue
  emit('change', `@default:${row.original}`)
  try {
    await invoke('select_default_checker', { name: row.original })
  } catch (e) {
    emit('change', previous ?? undefined)
    throwError('Failed to select default checker: ' + e)
  }
}

onMounted(async () => {
  files.value = await invoke<string[]>('get_default_checker_files')
})
</script>
