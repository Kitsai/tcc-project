import type { Problem } from "~/types/problem/problem";

export interface PolygonImportResult {
  problem: Problem;
  warnings: string[];
}