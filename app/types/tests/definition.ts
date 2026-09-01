export interface TestDefinition {
  id: number,
  testType: TestType,
  content: string,
  example: boolean,
  description: string
}

export type TestType = 'Manual' | 'Script'

export interface TestDefinitionCreateDto {
  id: number,
  testType: TestType,
  content: string,
  example: boolean,
  description: string
}

export interface TestDefinitionEditDto {
  id: number,
  testType: TestType,
  content: string,
  example: boolean,
  description: string
}

export interface GeneratedFile {
  name: string,
  content: string
}

export type PreviewOutcome =
  | { kind: 'Single', content: string }
  | { kind: 'Multiple', files: GeneratedFile[] }
