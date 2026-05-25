# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

A desktop application for authoring competitive programming problems. Built with Tauri v2 (Rust backend) + Nuxt 4 / Vue 3 (frontend). The app lets users create problems with statements (LaTeX), checker/validator scripts, and test cases.

## Commands

```bash
# Run the full app (Nuxt dev server + Tauri window)
pnpm tauri dev

# Run only the Nuxt frontend (no Tauri)
pnpm dev

# Build for production
pnpm tauri build

# Rust: build backend only
cd src-tauri && cargo build

# Rust: run tests
cd src-tauri && cargo test

# Rust: run a single test
cd src-tauri && cargo test <test_name>
```

## Architecture

### Frontend (`app/`)

Nuxt 4 SPA (SSR disabled). Key conventions:
- **Tauri calls** always go through `useTauri()` composable (`app/composables/tauri.ts`) which wraps `invoke`, `listen`, and `emit`.
- **State** is managed in Pinia stores (`app/stores/`): `useProblems` holds the open problem, `useSettingsStore` holds app settings.
- **Pages**: `index.vue` (home / open problem), `problem.vue` (layout for Statement/Checker/Validator tabs via nested routes), `settings.vue`.
- **Monaco Editor** (`app/components/CodeEditor.vue`) reads/writes files via `read_file_content`/`write_file_content` Tauri commands. After mounting, it calls `initLsp()` to connect to the language server.
- **LSP client** (`app/composables/useLsp.ts`) is a manual implementation: it calls `lsp_start` to get a WebSocket port, then handles the JSON-RPC protocol directly (hover, completion, diagnostics) without using the `monaco-languageclient` library.
- **Problem statement editor** (`app/components/TexEditor.vue`) uses TipTap + `@tiptap/extension-mathematics` (KaTeX) for inline LaTeX rendering.

### Backend (`src-tauri/src/`)

State singletons managed by Tauri (`manage()`):
- `ProblemManager` — holds the currently open `Problem` in an `RwLock`
- `LspBridge` — manages spawned LSP server processes
- `LspRegistry` — maps language IDs to server implementations
- `AppSettings` — persisted to `~/.config/tcc-project/settings.json`
- `SimpleRunner` — executes code with timeout/memory limits

**Problem module** (`src-tauri/src/problem/`):
- Problems live on disk as a directory. The `.prblm` file (JSON) is the `ProblemDefinition` (name, checker path, validator path).
- Directory structure inside each problem folder: `files/`, `solutions/`, `tests/validator/`, `tests/checker/`, `statement/`.
- Persistence uses the `Persistant` trait. Structs that derive `Serialize`/`Deserialize` and implement `SerdePersistant` get JSON load/save for free via the blanket impl in `util.rs`.

**LSP module** (`src-tauri/src/lsp/`):
- `LspBridge::start_for_language()` spawns a language server process (clangd or pylsp), binds a random TCP port, and accepts exactly one WebSocket connection per server instance.
- The bridge proxies bidirectionally: WebSocket messages → LSP stdin (adds `Content-Length` header), LSP stdout → WebSocket (strips headers).
- LSP servers are keyed by `(language_id, workspace_dir)` — one server instance per language per workspace.
- Bundled binaries: `binaries/linux-x64/pylsp` (PyInstaller bundle), clangd resolved from PATH.
- C++ headers bundled at `src-tauri/resources/includes/` (includes `testlib.h` and `bits/stdc++.h`). On startup, a `compile_flags.txt` is written to `~/.tcc-project/` to point clangd at them.

**Commands** (`src-tauri/src/commands/`):
- `problems.rs`: create/load problem, save statement, select checker/validator file, CRUD for validator tests.
- `lsp.rs`: `lsp_start` (returns WS port), `lsp_stop_all`.
- `files.rs`: read/write file content, list files in problem subdirectories, create/delete files.
- `settings.rs`: get/save settings, get app paths.
- `compile.rs`: check which languages are available.

### Adding a new Tauri command

1. Implement the function in the appropriate `src-tauri/src/commands/*.rs` file with `#[tauri::command]`.
2. Register it in `src-tauri/src/lib.rs` inside `tauri::generate_handler![]`.
3. Add a Tauri permission if needed in `src-tauri/capabilities/default.json`.
4. Call it from Vue with `const { invoke } = useTauri()` → `invoke<ReturnType>('command_name', { args })`.
