/** Composes a Script test's `content` field (`<generator> [args...]`) from
 *  its separate generator-file and args form fields. */
export function buildScriptContent(generatorFile: string, args: string): string {
  return [generatorFile, args].filter(Boolean).join(' ').trim()
}

/** Splits a Script test's `content` field back into its generator-file and
 *  args form fields, the inverse of `buildScriptContent`. */
export function parseScriptContent(content: string): { generatorFile: string, args: string } {
  const [first, ...rest] = content.trim().split(/\s+/)
  return { generatorFile: first ?? '', args: rest.join(' ') }
}