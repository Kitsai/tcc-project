<template>
  <div>
    <ProblemCheckerDefaultFilesSection :active-value="checkerValue" @change="onCheckerChange" />
    <USeparator />

    <div class="flex items-center gap-1 px-4 py-2">
      <span class="text-sm font-bold text-highlighted tracking-wide">Project Files</span>
      <div class="ml-auto flex items-center gap-2">
        <ButtonTextField label="Add File" :loading="loading" @submit="onAddFile" />
        <UTooltip text="Refresh">
          <UButton icon="i-lucide-refresh-cw" variant="ghost" size="sm" :loading="loading" @click="getFiles" />
        </UTooltip>
      </div>
    </div>

    <UTable :loading="loading" :data="files" :columns="columns" :ui="{ thead: 'hidden' }">
      <template #fileName-cell="{ row }">
        {{ row.original }}
      </template>

      <template #role-cell="{ row }">
        <USelect :model-value="roleFor(row.original)" :items="roleItems" variant="ghost" class="w-32"
          @update:model-value="(role: any) => onRoleChange(row.original, role)" />
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
import type { TableColumn } from '@nuxt/ui'

type FileRole = 'none' | 'checker' | 'validator' | 'generator'

const problems = useProblems()
const { invoke } = useTauri()
const { throwError } = useCustomToast()
const { openInEditor } = useProblemEditor()

const files = ref<string[]>([])
const loading = ref(false)

const checkerValue = computed(() => problems.currentProblem?.definition.checker)
const validatorValue = computed(() => problems.currentProblem?.definition.validator)

const roleItems = [
  { label: 'None', value: 'none' },
  { label: 'Checker', value: 'checker' },
  { label: 'Validator', value: 'validator' },
  { label: 'Generator', value: 'generator' },
]

const columns: TableColumn<string>[] = [
  { id: 'fileName', header: 'File Name', accessorFn: (row: string) => row },
  { id: 'role', header: 'Role' },
  { id: 'actions', header: '' },
]

function roleFor(file: string): FileRole {
  if (checkerValue.value === file) return 'checker'
  if (validatorValue.value === file) return 'validator'
  if (problems.currentProblem?.definition.generators?.includes(file)) return 'generator'
  return 'none'
}

function onCheckerChange(value: string | undefined) {
  if (!problems.currentProblem) return
  problems.currentProblem.definition.checker = value
}

async function onRoleChange(file: string, role: FileRole) {
  if (!problems.currentProblem) return

  const previousRole = roleFor(file)
  if (role === previousRole) return

  try {
    if (role === 'checker' || role === 'validator') {
      await invoke('select_problem_file', { fileType: role, file })
      problems.currentProblem.definition[role] = file
    } else if (role === 'generator') {
      await invoke('tag_generator_file', { file })
      problems.currentProblem.definition.generators ??= []
      if (!problems.currentProblem.definition.generators.includes(file)) {
        problems.currentProblem.definition.generators.push(file)
      }
    }

    if (previousRole === 'checker' || previousRole === 'validator') {
      await invoke('unselect_problem_file', { fileType: previousRole })
      problems.currentProblem.definition[previousRole] = undefined
    } else if (previousRole === 'generator') {
      await invoke('untag_generator_file', { file })
      const idx = problems.currentProblem.definition.generators?.indexOf(file) ?? -1
      if (idx >= 0) problems.currentProblem.definition.generators.splice(idx, 1)
    }
  } catch (e) {
    throwError('Failed to update role: ' + e)
  }
}

async function getFiles() {
  loading.value = true
  try {
    files.value = await invoke<string[]>('get_files')
  } catch (e) {
    console.error('[files.vue] Error in getFiles:', e)
  }
  loading.value = false
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
  const role = roleFor(filename)
  openInEditor(role === 'none' ? 'generator' : role, filename)
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
