export const useGeneratorFiles = () => {
  const problems = useProblems();

  return computed(() => problems.currentProblem?.definition.generators ?? []);
};
