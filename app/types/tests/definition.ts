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
