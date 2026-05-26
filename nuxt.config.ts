// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  devtools: { enabled: false },
  modules: ["@pinia/nuxt", "@nuxt/ui"],
  css: ["~/assets/css/main.css"],
  ssr: false,
  telemetry: false,
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
        "vscode-ws-jsonrpc",
        "@tauri-apps/api/path",
        "@tiptap/vue-3",
        "@tiptap/starter-kit",
        "@tiptap/extension-mathematics",
        "@tauri-apps/plugin-fs",
        "@tiptap/core",
        "@tauri-apps/plugin-clipboard-manager",
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
  app: {
    pageTransition: { name: "page", mode: "out-in" },
  },
});
