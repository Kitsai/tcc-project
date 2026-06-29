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
- **Pages**: `index.vue` (home / open problem), `problem.vue` (layout for Statement/Checker/Validator/Solution tabs via nested routes), `problem/editor.vue` (Monaco code editor), `settings.vue`.
- **Code editor navigation** goes through `useProblemEditor()` (`app/composables/useProblemEditor.ts`): `openInEditor(type, file)` pushes to `/problem/editor?type=&file=`. Used by `solution.vue` and `Problem/Files.vue`.
- **Monaco Editor** (`app/components/CodeEditor.vue`) reads/writes files via `read_file_content`/`write_file_content` Tauri commands. After mounting, it calls `initLsp()` to connect to the language server.
- **LSP client** (`app/composables/useLsp.ts`) is a manual implementation: it calls `lsp_start` to get a WebSocket port, then handles the JSON-RPC protocol directly (hover, completion, diagnostics) without using the `monaco-languageclient` library.
- **Problem statement editor** (`app/components/TexEditor.vue`) uses TipTap + `@tiptap/extension-mathematics` (KaTeX) for inline LaTeX rendering.

### Backend (`src-tauri/src/`)

State singletons managed by Tauri (`manage()`):
- `ProblemManager` — holds the currently open `Problem` in an `RwLock`
- `LspBridge` — manages spawned LSP server processes
- `LspRegistry` — maps language IDs to server implementations
- `AppSettings` — persisted to `~/.config/tcc-project/settings.json`
- `SimpleRunner` — executes code with timeout/memory limits; managed as `Arc<dyn Runner>` to allow swapping implementations

**Problem module** (`src-tauri/src/problem/`):
- Problems live on disk as a directory. The `.prblm` file (JSON) is the `ProblemDefinition` (`name`, `checker`, `validator`, `main_solution` — the last a fast-access pointer to the MAIN solution's filename, kept in sync by `change_tag`/`verify_solutions`).
- Directory structure inside each problem folder: `files/`, `solutions/`, `tests/validator/`, `tests/checker/`, `statement/`.
- Persistence uses the `Persistant` trait. Structs that derive `Serialize`/`Deserialize` and implement `SerdePersistant` get JSON load/save for free via the blanket impl in `util.rs`.
- **Solutions** (`solutions.rs`): each source file in `solutions/` has a sibling `<file>.desc` JSON holding a `SolutionDescription` (`file_name`, `tag`, `author`, `change_time`). `SolutionTag`: `Main`, `Accepted`, `WrongAnswer`, `TimeLimitExceeded`, `TimeLimitExceededOrAccepted`, `TimeLimitExceededOrMemoryLimitExceeded`, `MemoryLimitExceeded`, `None`. Load paths: `load_all` (read descs, no reconciliation) vs `verify_and_load` (reconcile descs against on-disk sources — create missing, delete orphaned). `change_tag` enforces a single-`Main` invariant (demotes the previous `Main` to `Accepted`).
- **Checker/Validator tests** (`checker.rs`, `validator.rs`): `CheckerTest`/`ValidatorTest` persisted one-file-per-test under `tests/checker/` and `tests/validator/`. `ValidatorTest::run_all` runs the validator against every test concurrently, emitting `validator_test_result`/`validator_test_error` events.

**LSP module** (`src-tauri/src/lsp/`):
- `LspBridge::start_for_language()` spawns a language server process (clangd or pylsp), binds a random TCP port, and accepts exactly one WebSocket connection per server instance.
- The bridge proxies bidirectionally: WebSocket messages → LSP stdin (adds `Content-Length` header), LSP stdout → WebSocket (strips headers).
- LSP servers are keyed by `(language_id, workspace_dir)` — one server instance per language per workspace.
- Bundled binaries: `binaries/linux-x64/pylsp` (PyInstaller bundle), clangd resolved from PATH.
- C++ headers bundled at `src-tauri/resources/includes/` (includes `testlib.h` and `bits/stdc++.h`). On startup, a `compile_flags.txt` is written to `~/.tcc-project/` to point clangd at them.

**Commands** (`src-tauri/src/commands/`):
- `problems.rs`: create/load problem, save statement, select checker/validator file.
- `validator.rs`: CRUD for validator tests, `run_validator_tests`.
- `solution.rs`: `get_solutions` (`load_all`), `change_tag` and `verify_solutions` (both also sync `definition.main_solution`), create/add/delete solution files.
- `lsp.rs`: `lsp_start` (returns WS port), `lsp_stop_all`.
- `files.rs`: read/write file content, list files in problem subdirectories, create/delete files.
- `settings.rs`: get/save settings, get app paths.
- `compile.rs`: check which languages are available.

### Adding a new Tauri command

1. Implement the function in the appropriate `src-tauri/src/commands/*.rs` file with `#[tauri::command]`.
2. Register it in `src-tauri/src/lib.rs` inside `tauri::generate_handler![]`.
3. Add a Tauri permission if needed in `src-tauri/capabilities/default.json`.
4. Call it from Vue with `const { invoke } = useTauri()` → `invoke<ReturnType>('command_name', { args })`.
