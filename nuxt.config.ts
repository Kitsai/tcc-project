// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  devtools: { enabled: false },
  modules: ["@pinia/nuxt", "@nuxt/ui", "nuxt-monaco-editor"],
  css: ["~/assets/css/main.css"],
  ssr: false,
  devServer: {
    port: 3000,
  },
  monacoEditor: {
    // @ts-ignore
    optimizeMonacoDeps: false,
  },
  vite: {
    clearScreen: false,
    envPrefix: ["VITE_", "TAURI_"],
    server: {
      strictPort: true,
      headers: {
        "Cache-Control": "no-store",
      },
    },
  },
  ignore: ["**/src-tauri/**"],
});