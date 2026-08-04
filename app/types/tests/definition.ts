export interface TestDefinition {
  id: number,
  testType: TestType,
  content: string,
  example: boolean,
  description: string
}

export type TestType = 'Manual' | 'Script'
