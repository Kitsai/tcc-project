# Recursos embutidos: como o app acha `testlib.h` e os checkers padrão

## O problema em uma frase

O app procura os arquivos que vêm dentro dele (`testlib.h`, `bits/stdc++.h`, os checkers padrão) a partir da
**pasta de onde o processo foi iniciado**. Isso só funciona dentro do repositório. No app instalado essa pasta é
outra, o caminho não existe, e o compilador responde `testlib.h: No such file or directory`.

## Conceitos

**Recurso embutido.** Arquivo que o instalador copia junto com o executável. Está listado em
`src-tauri/tauri.conf.json`, em `bundle.resources`:

```json
"resources": ["resources/includes/**/*", "resources/checkers/**/*"]
```

**Pasta de trabalho (`current_dir`).** Pasta "atual" do processo. Quem decide é quem abriu o programa: no
`pnpm tauri dev` é o `src-tauri`; com um atalho do Windows pode ser qualquer outra. **Não diz nada sobre onde
o app está instalado.**

**Pasta de recursos (`resource_dir`).** Pasta onde o Tauri colocou os recursos. Só o Tauri sabe qual é, e muda
por sistema.

## Onde os arquivos ficam em cada ambiente

| Ambiente | Pasta base | Includes |
|---|---|---|
| Desenvolvimento | `<repo>/src-tauri` | `<repo>/src-tauri/resources/includes/testlib.h` |
| Windows instalado | `%LOCALAPPDATA%\Ballooner` | `%LOCALAPPDATA%\Ballooner\resources\includes\testlib.h` (conferido) |
| Linux instalado | pasta de recursos do pacote | `<pasta de recursos>/resources/includes/testlib.h` (não conferido) |

A parte `resources/includes/...` é igual nos três. **Só muda a base.** É isso que a solução explora.

## O que o código faz hoje

`src-tauri/src/problem/files.rs`:

- `get_include_paths()` e `get_default_checkers_path()` partem de `std::env::current_dir()`:

  ```rust
  if current_dir.ends_with("src-tauri") { current_dir.join("resources/includes") }
  else { current_dir.join("src-tauri/resources/includes") }
  ```

  Uma heurística: "se estou no repositório, está aqui". Acerta no desenvolvimento, erra no instalado. O
  `if resource_path.exists()` descarta o caminho errado **em silêncio**, e por isso não há erro visível, só o
  compilador reclamando depois.

- `lsp/binary_resolver.rs` já parte da pasta de recursos do Tauri (o jeito certo):

  ```rust
  let context = tauri::generate_context!();
  resource_dir(context.package_info(), &Default::default())
  ```

  Mas chama a macro a cada uso, e a macro embute o frontend no binário de novo.

### Quem é afetado
- A lista de `-I` do `g++` (compilar checker, validador, solução) → falha em `testlib.h`.
- O `compile_flags.txt` do clangd (autocompletar) → sem as bibliotecas.
- A lista de checkers padrão e o `@default:mojdiff.cpp` (usado ao importar do MOJ e ao exportar).

## A solução

Uma função só, que tenta dois pontos de partida e devolve o primeiro que existir:

```rust
pub fn bundled_resource(relative: &str) -> Option<PathBuf> {
    // 1. instalado: <pasta de recursos do Tauri>/resources/<relative>
    // 2. desenvolvimento: <CARGO_MANIFEST_DIR>/resources/<relative>
    // devolve o primeiro que existir
}
```

- `relative` é `"includes"` ou `"checkers"`.
- `env!("CARGO_MANIFEST_DIR")` é o caminho do `src-tauri`, **gravado na compilação**. Diferente do `current_dir()`,
  não muda conforme quem abriu o app. Num app instalado ele aponta para uma pasta que não existe na máquina do
  usuário, e o `exists()` o ignora, que é o desejado.
- `get_include_paths` chama `bundled_resource("includes")`; `get_default_checkers_path` chama
  `bundled_resource("checkers")`. Somem os dois `if/else` com `src-tauri`.

### De onde vem a pasta de recursos

Calculada **uma vez**, em `lib.rs::run()`, e guardada num `OnceLock`:

1. `let context = tauri::generate_context!();`   (a macro, uma só vez)
2. calcular a pasta com `resource_dir(context.package_info(), ...)` e guardar no `OnceLock`;
3. montar o registro do LSP (`get_include_paths()` já encontra o valor);
4. `Builder ... .run(context)`.

A ordem importa: hoje o `get_include_paths()` roda **antes** de o `Builder` existir, então o valor precisa estar
guardado antes dele. Depois, o `BinaryResolver` pode ler o mesmo `OnceLock` e deixar de chamar a macro.

## Passo a passo para reescrever

1. Escrever `bundled_resource` e o `OnceLock`.
2. Trocar `get_include_paths` e `get_default_checkers_path` para usá-la.
3. Em `lib.rs`: contexto uma vez, guardar a pasta, depois o resto. Ajustar o `BinaryResolver` (opcional).
4. Teste: `cargo test` roda de dentro de `src-tauri`, então `bundled_resource("includes")` deve devolver um
   caminho que contém `testlib.h` (cobre o caminho de desenvolvimento).
5. Nova tag, instalar, e compilar um checker no Windows (cobre o caminho instalado).

## Armadilhas

- **Prefixo `\\?\` no Windows.** A pasta de recursos pode vir como `\\?\C:\...`, e o `g++` do MinGW costuma recusar
  isso em `-I`. O código já troca `\` por `/`, mas o prefixo sobraria. Se o erro persistir depois do conserto,
  tire-o antes de montar o `-I`. (Possibilidade, ainda não observada.)
- **`generate_context!()` duplicado.** Evite chamá-lo em outro lugar. Um só.
- **Silêncio do `exists()`.** Se `bundled_resource` devolver `None`, ninguém avisa. Vale um `log::warn!` dizendo
  qual recurso não foi achado e onde procurou: facilita o diagnóstico no instalado.
- **`pylsp`.** O `BinaryResolver` procura em `<recursos>/binaries/<plataforma>/`, mas `binaries/` **não está** em
  `bundle.resources`. No app instalado o autocompletar de Python cai no `pylsp` do sistema. Problema separado.

## Como verificar

- **Desenvolvimento:** o teste do passo 4.
- **Instalado (Windows):**
  ```powershell
  dir $env:LOCALAPPDATA\Ballooner\resources -Recurse | select -First 30 FullName
  ```
  Deve listar `includes\testlib.h` e `checkers\...`. Depois, compilar um checker pelo app.
