export interface ValidatorTest {
  id: number,
  input: string,
  expected: ValidatorTestResult,
  actual: ValidatorTestResult,
}

export type ValidatorTestResult = "VALID" | "INVALID" | "";

export interface ValidatorTestCreateDto {
  id: number,
  mult: boolean,
  input: string,
  verdict: string,
}


export interface ValidatorTestEditDto {
  id: number,
  input: string,
  verdict: string,
}
