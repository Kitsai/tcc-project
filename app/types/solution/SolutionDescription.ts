export interface SolutionDescription {
  file_name: string,
  tag: SolutionTag,
  author?: string,
  change_time: string
}

export type SolutionTag = "MAIN" |
  "ACCEPTED" |
  "WRONG_ANSWER" |
  "TIME_LIMIT_EXCEEDED" |
  "TIME_LIMIT_EXCEEDED_OR_ACCEPTED" |
  "TIME_LIMIT_EXCEEDED_OR_MEMORY_LIMIT_EXCEEDED" |
  "MEMORY_LIMIT_EXCEEDED" |
  "";

export const SOLUTION_TAGS: SolutionTag[] = [
  "MAIN",
  "ACCEPTED",
  "WRONG_ANSWER",
  "TIME_LIMIT_EXCEEDED",
  "TIME_LIMIT_EXCEEDED_OR_ACCEPTED",
  "TIME_LIMIT_EXCEEDED_OR_MEMORY_LIMIT_EXCEEDED",
  "MEMORY_LIMIT_EXCEEDED",
];

export function tag_to_text(tag: SolutionTag): string {
  switch (tag) {
    case "MAIN": return "Main correct solution";
    case "ACCEPTED": return "Correct";
  }

  return tag.charAt(0) + tag.slice(1).toLowerCase().replaceAll("_", " ");
}

export function success_tag(tag: SolutionTag) {
  return tag === "MAIN" || tag === "ACCEPTED"
};
