export interface ValidatorTest {
  id: number,
  input: string,
  expected: ValidatorTestResult,
  actual: ValidatorTestResult,
}

export type ValidatorTestResult = "VALID" | "INVALID" | "";
