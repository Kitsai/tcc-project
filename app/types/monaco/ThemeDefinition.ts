import { editor } from "monaco-editor"

export interface VSCodeTheme {
  name?: string
  type?: 'dark' | 'light' | 'hc'
  colors?: { [key: string]: string | null }
  tokenColors?: Array<{
    name?: string
    scope?: string | string[]
    settings: {
      foreground?: string | null
      background?: string | null
      fontStyle?: string | null
    }
  }>
}

export interface ThemeDefinition {
  name: string
  themeData: editor.IStandaloneThemeData
}

export function convertVSCodeToMonaco(vscodeTheme: VSCodeTheme): editor.IStandaloneThemeData {
  const rules: editor.ITokenThemeRule[] = []
  const colors: { [key: string]: string } = {}

  // Process UI colors, filtering out nulls
  if (vscodeTheme.colors) {
    for (const key in vscodeTheme.colors) {
      const color = vscodeTheme.colors[key]
      if (typeof color === 'string') {
        colors[key] = color
      }
    }
  }

  if (vscodeTheme.tokenColors) {
    for (const entry of vscodeTheme.tokenColors) {
      if (!entry.settings) continue

      const foreground = entry.settings.foreground || undefined
      const background = entry.settings.background || undefined
      const fontStyle = entry.settings.fontStyle || undefined

      // If no scope, it's a global setting (like "Global settings" in Night Owl)
      if (!entry.scope) {
        // We can't easily map global background/foreground to a "token" rule
        // so we ensure they are in the colors object if not already present
        if (foreground && !colors['editor.foreground']) colors['editor.foreground'] = foreground
        if (background && !colors['editor.background']) colors['editor.background'] = background
        continue
      }

      const scopes = Array.isArray(entry.scope) ? entry.scope : [entry.scope]
      for (const scope of scopes) {
        rules.push({
          token: scope,
          foreground: foreground as string,
          background: background as string,
          fontStyle: fontStyle as string
        })
      }
    }
  }

  return {
    base: vscodeTheme.type === 'dark' ? 'vs-dark' : (vscodeTheme.type === 'hc' ? 'hc-black' : 'vs'),
    inherit: true,
    rules: rules,
    colors: colors
  }
}
