// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  devtools: { enabled: false },
  modules: ["@pinia/nuxt", "@nuxt/ui"],
  css: ["~/assets/css/main.css"],
  ssr: false,
  devServer: {
    port: 3000,
  },
  vite: {
    optimizeDeps: {
      include: [
        "monaco-editor",
        "@tauri-apps/api",
        "@tauri-apps/api/core",
        "@tauri-apps/api/event",
        "@tauri-apps/plugin-dialog",
        "vscode-ws-jsonrpc"
      ],
    },
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
