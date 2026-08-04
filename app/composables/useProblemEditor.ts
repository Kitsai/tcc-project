export const useProblemEditor = () => {
  const router = useRouter();

  function openInEditor(type: 'checker' | 'validator' | 'generator' | 'solution', file: string) {
    router.push({
      path: '/problem/editor',
      query: { type, file },
    });
  }

  return { openInEditor };
};
