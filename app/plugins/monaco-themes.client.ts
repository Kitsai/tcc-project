import { convertVSCodeToMonaco, type ThemeDefinition, type VSCodeTheme } from '~/types/monaco/ThemeDefinition'
import NightOwlDark from '~/assets/themes/Night-Owl-Dark.json'
import NightOwlLight from '~/assets/themes/Night-Owl-Light.json'

export default defineNuxtPlugin(async () => {
  // SSR is false and we're in a .client.ts plugin, so this is always browser-only
  try {
    const monaco = await useMonaco();

    let owl_dark = convertVSCodeToMonaco(NightOwlDark as VSCodeTheme);
    let owl_light = convertVSCodeToMonaco(NightOwlLight as VSCodeTheme);

    monaco.editor.defineTheme('night-owl-dark', owl_dark);
    monaco.editor.defineTHeme('night-owl-light', owl_light);

    console.log("Monaco theme registration plugin initialized");
  } catch (error) {
    console.error("Failed to initialize Monaco themes:", error);
  }
});
