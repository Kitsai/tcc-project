export interface Problem {
  definition: ProblemDefinition,
  stmt: ProblemStatement,
}

export interface ProblemDefinition {
  name: string
}

export interface ProblemStatement {
  name: string,
  legend: string,
  input: string,
  output: string,
  notes: string,
  tutorial: string,
}
