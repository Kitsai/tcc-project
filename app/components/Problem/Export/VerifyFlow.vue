<template>
  <div class="flex flex-col gap-6 max-w-3xl">
    <div class="flex items-start justify-between gap-4">
      <div>
        <h2 class="text-lg font-semibold">Export Problem</h2>
        <p class="text-sm text-muted">
          Verifies the problem, then packages it for export: generate tests, validate them, run the main
          solution to produce expected outputs, check every result with the checker, then build the package.
        </p>
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <UButton label="Run Verification" variant="subtle" icon="i-lucide-check-check" :loading="activeRun === 'verify'"
          :disabled="isRunning" @click="runPipeline('verify')" />
        <UButton label="Export" icon="i-lucide-package" :loading="activeRun === 'export'" :disabled="isRunning"
          @click="runPipeline('export')" />
      </div>
    </div>

    <UStepper :items="stepperItems" :model-value="activeKey" disabled orientation="horizontal" />
  </div>
</template>

<script setup lang="ts">
import type { VerificationStage, VerificationStageKey } from "~/types/export/verification";

type RunMode = "verify" | "export";

const { throwError, throwSuccess } = useCustomToast();

const activeRun = ref<RunMode | null>(null);
const isRunning = computed(() => activeRun.value !== null);

const stages = ref<VerificationStage[]>([
  {
    key: "generate",
    title: "Generate Tests",
    description: "Run the generator scripts to produce test inputs.",
    status: "pending",
    error: null,
  },
  {
    key: "validate",
    title: "Validate",
    description: "Check every generated input against the validator.",
    status: "pending",
    error: null,
  },
  {
    key: "solve",
    title: "Run Main Solution",
    description: "Produce the expected output for every test.",
    status: "pending",
    error: null,
  },
  {
    key: "check",
    title: "Verify with Checker",
    description: "Compare every output against the expected one.",
    status: "pending",
    error: null,
  },
  {
    key: "export",
    title: "Export Package",
    description: "Package the problem for export.",
    status: "pending",
    error: null,
  },
]);

const activeKey = computed<VerificationStageKey>(() => {
  const running = stages.value.find(s => s.status === "running");
  if (running) return running.key;

  const failed = stages.value.find(s => s.status === "error");
  if (failed) return failed.key;

  const lastDone = [...stages.value].reverse().find(s => s.status === "success");
  return lastDone?.key ?? stages.value[0]!.key;
});

function iconFor(status: VerificationStage["status"]) {
  switch (status) {
    case "running": return "i-lucide-loader-circle";
    case "success": return "i-lucide-check";
    case "error": return "i-lucide-x";
    default: return "i-lucide-circle-dashed";
  }
}

const stepperItems = computed(() => stages.value.map(stage => ({
  value: stage.key,
  title: stage.title,
  description: stage.error ?? stage.description,
  icon: iconFor(stage.status),
  ui: stage.status === "running" ? { icon: "animate-spin" } : undefined,
})));

function resetStages() {
  for (const stage of stages.value) {
    stage.status = "pending";
    stage.error = null;
  }
}

// TODO: wire these up to real Tauri commands once the backend exists.
async function runGenerateStage() {
  throw new Error("Not implemented yet");
}

async function runValidateStage() {
  throw new Error("Not implemented yet");
}

async function runSolveStage() {
  throw new Error("Not implemented yet");
}

async function runCheckStage() {
  throw new Error("Not implemented yet");
}

async function runExportStage() {
  throw new Error("Not implemented yet");
}

const runners: Record<VerificationStageKey, () => Promise<void>> = {
  generate: runGenerateStage,
  validate: runValidateStage,
  solve: runSolveStage,
  check: runCheckStage,
  export: runExportStage,
};

async function runPipeline(mode: RunMode) {
  if (isRunning.value) return;

  activeRun.value = mode;
  resetStages();

  const stagesToRun = mode === "verify"
    ? stages.value.filter(s => s.key !== "export")
    : stages.value;

  for (const stage of stagesToRun) {
    stage.status = "running";
    try {
      await runners[stage.key]();
      stage.status = "success";
    } catch (e) {
      stage.status = "error";
      stage.error = e instanceof Error ? e.message : String(e);
      throwError(`${stage.title} failed: ${stage.error}`);
      activeRun.value = null;
      return;
    }
  }

  activeRun.value = null;
  throwSuccess(mode === "verify" ? "All tests verified successfully!" : "Export completed successfully!");
}
</script>