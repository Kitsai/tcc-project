import Mathematics from "@tiptap/extension-mathematics";
import { InputRule } from "@tiptap/core";

export const MathematicsWithInline = Mathematics.extend({
  addInputRules() {
    // Mathematics is a wrapper and doesn't have addInputRules itself, 
    // but its sub-extensions might. We safely check here.
    const parentInputRules = this.parent?.() || []

    return [
      ...parentInputRules,
      // Rule for standard inline math: $latex$
      new InputRule({
        find: /(?<!\$)(\$([^$\s$][^$\n]*?[^$\s$])\$)(?!\$)/,
        handler: ({ state, range, match }) => {
          const type = state.schema.nodes['inlineMath'];
          if (!type) return;

          const latex = match[2]
          const { tr } = state

          if (latex) {
            tr.replaceWith(range.from, range.to, type.create({ latex }))
          }
        },
      }),
      // Rule for single character math: $x$
      new InputRule({
        find: /(?<!\$)(\$([^$\s$])\$)(?!\$)/,
        handler: ({ state, range, match }) => {
          const type = state.schema.nodes['inlineMath'];
          if (!type) return;

          const latex = match[2]
          const { tr } = state

          if (latex) {
            tr.replaceWith(range.from, range.to, type.create({ latex }))
          }
        },
      })
    ]
  }
})
