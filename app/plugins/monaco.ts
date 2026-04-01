import { convertVSCodeToMonaco, type VSCodeTheme } from "~/types/monaco/ThemeDefinition";
import NightOwlDark from "~/assets/themes/Night-Owl-Dark.json";
import NightOwlLight from "~/assets/themes/Night-Owl-Light.json";
import * as monaco from 'monaco-editor';

import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker';
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker';
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker';

export default defineNuxtPlugin((nuxtApp) => {
  // Define workers - Standard Monaco way
  self.MonacoEnvironment = {
    getWorker(_, label) {
      if (label === 'json') {
        return new jsonWorker();
      }
      if (label === 'css' || label === 'scss' || label === 'less') {
        return new cssWorker();
      }
      if (label === 'html' || label === 'handlebars' || label === 'razor') {
        return new htmlWorker();
      }
      if (label === 'typescript' || label === 'javascript') {
        return new tsWorker();
      }
      return new editorWorker();
    },
  };

  // Register themes
  const owl_dark = convertVSCodeToMonaco(NightOwlDark as VSCodeTheme);
  const owl_light = convertVSCodeToMonaco(NightOwlLight as VSCodeTheme);

  monaco.editor.defineTheme("night-owl-dark", owl_dark);
  monaco.editor.defineTheme("night-owl-light", owl_light);
});
