export default defineNuxtRouteMiddleware((to, from) => {
  const problems = useProblems();

  problems.currentProblem = null;
});
