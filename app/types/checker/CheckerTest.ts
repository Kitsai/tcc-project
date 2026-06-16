export interface CheckerTest {
  id: number,
  input: string,
  output: string,
  answer: string,
  expected: CheckerVerdict,
  actual: CheckerVerdict,
  comment: string,
}

export type CheckerVerdict = "OK" | "WRONG_ANSWER" | "PRESENTATION_ERROR" | "CRASHED" | "";

export interface CheckerTestCreateDto {
  id: number,
  mult: boolean,
  input: string,
  output: string,
  answer: string,
  verdict: string,
}
