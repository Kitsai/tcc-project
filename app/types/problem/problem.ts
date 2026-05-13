export interface Problem {
  path: string,
  definition: ProblemDefinition,
  stmt: ProblemStatement,
}

export interface ProblemDefinition {
  name: string,
  checker: string | undefined,
  validator: string | undefined,
}

export interface ProblemStatement {
  name: string,
  legend: string,
  input: string,
  output: string,
  notes: string,
  tutorial: string,
}
