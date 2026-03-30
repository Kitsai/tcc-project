import { convertVSCodeToMonaco, type VSCodeTheme } from "~/types/monaco/ThemeDefinition";
import NightOwlDark from "~/assets/themes/Night-Owl-Dark.json";
import NightOwlLight from "~/assets/themes/Night-Owl-Light.json";
import * as monaco from 'monaco-editor';

export default defineNuxtPlugin((nuxtApp) => {
  // Register themes
  const owl_dark = convertVSCodeToMonaco(NightOwlDark as VSCodeTheme);
  const owl_light = convertVSCodeToMonaco(NightOwlLight as VSCodeTheme);

  monaco.editor.defineTheme("night-owl-dark", owl_dark);
  monaco.editor.defineTheme("night-owl-light", owl_light);
});
