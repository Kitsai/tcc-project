export type VerificationStageKey = "generate" | "validate" | "solve" | "check" | "export";

export type VerificationStageStatus = "pending" | "running" | "success" | "error";

export interface VerificationStage {
  key: VerificationStageKey;
  title: string;
  description: string;
  status: VerificationStageStatus;
  error: string | null;
}
