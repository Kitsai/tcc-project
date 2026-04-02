import * as monaco from 'monaco-editor';

/**
 * Manual LSP Bridge for Monaco
 * 
 * This avoids monaco-languageclient/vscode-api to keep things lean and
 * avoid permission/clipboard/storage errors in Tauri.
 */
export const useLsp = () => {
  const { invoke } = useTauri();
  let socket: WebSocket | null = null;
  let nextId = 1;
  const pendingRequests = new Map<number, { resolve: (val: any) => void; reject: (err: any) => void }>();

  const sendRequest = (method: string, params: any) => {
    return new Promise((resolve, reject) => {
      if (!socket || socket.readyState !== WebSocket.OPEN) {
        return reject(new Error("LSP Socket not open"));
      }
      const id = nextId++;
      pendingRequests.set(id, { resolve, reject });
      socket.send(JSON.stringify({ jsonrpc: "2.0", id, method, params }));
    });
  };

  const sendNotification = (method: string, params: any) => {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ jsonrpc: "2.0", method, params }));
    }
  };

  const pathToUri = (path: string) => {
    const normalized = path.replace(/\\/g, '/');
    return normalized.startsWith('/') ? `file://${normalized}` : `file:///${normalized}`;
  };

  const initLsp = async (languageId: string, editor: monaco.editor.IStandaloneCodeEditor) => {
    // For development, we use a fixed project root.
    const projectRoot = "/home/Kitsai/coding/tcc/tcc-project";

    // 1. Get port from Rust
    const port = await invoke<number>("lsp_start", {
      languageId,
      workspaceDir: projectRoot
    });

    const url = `ws://127.0.0.1:${port}`;
    console.log(`[LSP-Manual] Connecting to ${url}`);
    socket = new WebSocket(url);

    socket.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      if (msg.id !== undefined && pendingRequests.has(msg.id)) {
        const { resolve, reject } = pendingRequests.get(msg.id)!;
        pendingRequests.delete(msg.id);
        if (msg.error) reject(msg.error);
        else resolve(msg.result);
      }

      // Handle diagnostics
      if (msg.method === "textDocument/publishDiagnostics") {
        const diagnostics = msg.params.diagnostics;
        const model = editor.getModel();
        if (model) {
          monaco.editor.setModelMarkers(model, "lsp", diagnostics.map((d: any) => ({
            startLineNumber: d.range.start.line + 1,
            startColumn: d.range.start.character + 1,
            endLineNumber: d.range.end.line + 1,
            endColumn: d.range.end.character + 1,
            message: d.message,
            severity: d.severity === 1 ? monaco.MarkerSeverity.Error : monaco.MarkerSeverity.Warning
          })));
        }
      }
    };

    socket.onopen = async () => {
      console.log("[LSP-Manual] Connected. Initializing...");

      const fileUri = pathToUri(`${projectRoot}/test.cpp`);

      // LSP Initialize
      await sendRequest("initialize", {
        processId: null,
        rootUri: pathToUri(projectRoot),
        capabilities: {
          textDocument: {
            hover: { contentFormat: ["markdown", "plaintext"] },
            completion: { completionItem: { snippetSupport: true } },
            publishDiagnostics: { relatedInformation: true }
          }
        }
      });
      sendNotification("initialized", {});

      // Open Document
      const model = editor.getModel();
      if (model) {
        sendNotification("textDocument/didOpen", {
          textDocument: {
            uri: fileUri,
            languageId,
            version: 1,
            text: model.getValue()
          }
        });

        // Sync changes
        let version = 1;
        model.onDidChangeContent(() => {
          version++;
          sendNotification("textDocument/didChange", {
            textDocument: { uri: fileUri, version },
            contentChanges: [{ text: model.getValue() }]
          });
        });
      }

      // Register Hover Provider
      monaco.languages.registerHoverProvider(languageId, {
        provideHover: async (model, position) => {
          const res: any = await sendRequest("textDocument/hover", {
            textDocument: { uri: fileUri },
            position: { line: position.lineNumber - 1, character: position.column - 1 }
          });
          if (!res || !res.contents) return null;
          return {
            contents: Array.isArray(res.contents) ? res.contents : [res.contents]
          };
        }
      });

      // Register Completion Provider
      monaco.languages.registerCompletionItemProvider(languageId, {
        triggerCharacters: ['.', ':', '>', '#'],
        provideCompletionItems: async (model, position) => {
          const res: any = await sendRequest("textDocument/completion", {
            textDocument: { uri: fileUri },
            position: { line: position.lineNumber - 1, character: position.column - 1 }
          });
          if (!res) return null;
          const items = Array.isArray(res) ? res : res.items;
          return {
            suggestions: items.map((item: any) => ({
              label: item.label,
              kind: item.kind || monaco.languages.CompletionItemKind.Function,
              insertText: item.insertText || item.label,
              insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
              detail: item.detail,
              documentation: item.documentation
            }))
          };
        }
      });

      console.log("[LSP-Manual] Handshake successful. Providers registered!");
    };
  };

  return { initLsp };
};
