export interface ExecutionInfo {
  stdout: string;
  stderr: string;
  execution_time: {
    secs: number;
    nanos: number;
  };
}

export type ExecutionError =
  | { TLE: { secs: number; nanos: number } }
  | { ME: number }
  | { OTHER: string };

export type ExecutionResult = ExecutionInfo | ExecutionError;

