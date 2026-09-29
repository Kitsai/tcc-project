export interface ExportManifestEntry {
  finalId: number;
  sourceTestId: number;
  example: boolean;
}

export interface ExportManifest {
  tests: ExportManifestEntry[];
}