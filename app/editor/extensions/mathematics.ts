import { Mathematics, migrateMathStrings } from "@tiptap/extension-mathematics";
import { InputRule } from "@tiptap/core";

export const MathematicsWithInline = Mathematics.extend({
  onCreate() {
    // Automatically convert $...$ and $$...$$ in the initial content
    if (this.editor) {
      migrateMathStrings(this.editor);
    }
  },

  addInputRules() {
    const type = this.editor.schema.nodes['inlineMath'];
    if (!type) return [];

    return [
      // Rule for standard inline math: $latex$
      new InputRule({
        find: /(?<!\$)(\$([^$\s$][^$\n]*?[^$\s$])\$)(?!\$)/,
        handler: ({ state, range, match }) => {
          const latex = match[2]
          if (latex) {
            state.tr.replaceWith(range.from, range.to, type.create({ latex }))
          }
        },
      }),
      // Rule for single character math: $x$
      new InputRule({
        find: /(?<!\$)(\$([^$\s$])\$)(?!\$)/,
        handler: ({ state, range, match }) => {
          const latex = match[2]
          if (latex) {
            state.tr.replaceWith(range.from, range.to, type.create({ latex }))
          }
        },
      })
    ]
  }
})

export { migrateMathStrings };
