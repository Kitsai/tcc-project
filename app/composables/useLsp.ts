import * as monaco from 'monaco-editor';

/**
 * Manual LSP Bridge for Monaco
 */
export const useLsp = () => {
  const { invoke } = useTauri();

  const pathToUri = (path: string) => {
    const normalized = path.replace(/\\/g, '/');
    return normalized.startsWith('/') ? `file://${normalized}` : `file:///${normalized}`;
  };

  const initLsp = async (filePath: string, workspaceDir: string, editor: monaco.editor.IStandaloneCodeEditor) => {
    // 1. Determine Language from Path
    const ext = filePath.split('.').pop()?.toLowerCase();
    const languageId = ext === 'py' ? 'python' : 'cpp';
    
    const fileUri = pathToUri(filePath);

    console.log(`[LSP-Manual] Initializing for ${languageId} at ${workspaceDir}`);

    // State for THIS specific connection
    let socket: WebSocket | null = null;
    let nextId = 1;
    const pendingRequests = new Map<number, { resolve: (val: any) => void; reject: (err: any) => void }>();

    // RPC Helpers
    const sendRequest = (method: string, params: any) => {
      return new Promise((resolve, reject) => {
        if (!socket || socket.readyState !== WebSocket.OPEN) return reject(new Error("LSP Socket not open"));
        const id = nextId++;
        pendingRequests.set(id, { resolve, reject });
        const msg = { jsonrpc: "2.0", id, method, params };
        console.log(`[LSP-Request] ${method}`, msg);
        socket.send(JSON.stringify(msg));
      });
    };

    const sendNotification = (method: string, params: any) => {
      if (socket && socket.readyState === WebSocket.OPEN) {
        const msg = { jsonrpc: "2.0", method, params };
        console.log(`[LSP-Notify] ${method}`, msg);
        socket.send(JSON.stringify(msg));
      }
    };

    // 2. Get port from Rust
    const port = await invoke<number>("lsp_start", { 
      languageId,
      workspaceDir 
    });
    
    const url = `ws://127.0.0.1:${port}`;
    socket = new WebSocket(url);

    socket.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      if (msg.method !== "textDocument/publishDiagnostics") {
        console.log(`[LSP-Response]`, msg);
      }
      
      if (msg.id !== undefined && pendingRequests.has(msg.id)) {
        const { resolve, reject } = pendingRequests.get(msg.id)!;
        pendingRequests.delete(msg.id);
        if (msg.error) reject(msg.error);
        else resolve(msg.result);
      }
      
      // Diagnostics
      if (msg.method === "textDocument/publishDiagnostics") {
        const model = editor.getModel();
        if (model) {
          monaco.editor.setModelMarkers(model, "lsp", msg.params.diagnostics.map((d: any) => ({
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
      console.log(`[LSP-Manual] WebSocket Open for ${languageId}`);
      
      const rootUri = pathToUri(workspaceDir);
      // Handshake
      console.log(`[LSP-Manual] Sending initialize for ${languageId}`);
      const initResult: any = await sendRequest("initialize", {
        processId: 0,
        rootPath: workspaceDir,
        rootUri: rootUri,
        capabilities: {
          textDocument: {
            hover: { contentFormat: ["markdown", "plaintext"] },
            completion: { 
              completionItem: { 
                snippetSupport: true,
                documentationFormat: ["markdown", "plaintext"]
              } 
            },
            publishDiagnostics: { relatedInformation: true }
          }
        }
      });
      sendNotification("initialized", {});

      // For pylsp, it often helps to send a configuration notification
      if (languageId === 'python') {
        sendNotification("workspace/didChangeConfiguration", {
          settings: {
            pylsp: {
              plugins: {
                jedi_completion: { 
                  enabled: true,
                  include_params: true,
                  include_class_objects: true,
                  fuzzy: true
                },
                jedi_hover: { enabled: true },
                jedi_references: { enabled: true },
                jedi_symbols: { enabled: true },
              }
            }
          }
        });
      }

      const serverTriggerChars = initResult?.capabilities?.completionProvider?.triggerCharacters || ['.'];
      console.log(`[LSP-Manual] Server trigger characters for ${languageId}:`, serverTriggerChars);

      // Sync Document
      const model = editor.getModel();
      if (model) {
        sendNotification("textDocument/didOpen", {
          textDocument: { uri: fileUri, languageId, version: 1, text: model.getValue() }
        });

        let version = 1;
        const changeSub = model.onDidChangeContent(() => {
          version++;
          sendNotification("textDocument/didChange", {
            textDocument: { uri: fileUri, version },
            contentChanges: [{ text: model.getValue() }]
          });
        });

        // Ensure we cleanup the socket if the editor/component dies
        editor.onDidDispose(() => {
          console.log(`[LSP-Manual] Editor disposed, closing socket for ${languageId}`);
          changeSub.dispose();
          socket?.close();
        });
      }

      console.log(`[LSP-Manual] Registering providers for ${languageId}`);

      const hoverProv = monaco.languages.registerHoverProvider(languageId, {
        provideHover: async (model, position) => {
          if (model.uri.toString() !== fileUri) {
            console.log(`[LSP-Hover] URI Mismatch: ${model.uri.toString()} !== ${fileUri}`);
            return null;
          }
          
          try {
            const res: any = await sendRequest("textDocument/hover", {
              textDocument: { uri: fileUri },
              position: { line: position.lineNumber - 1, character: position.column - 1 }
            });
            if (!res || !res.contents) return null;
            return { contents: Array.isArray(res.contents) ? res.contents : [res.contents] };
          } catch (e) {
            console.error(`[LSP-Hover] Error:`, e);
            return null;
          }
        }
      });

      const compProv = monaco.languages.registerCompletionItemProvider(languageId, {
        triggerCharacters: serverTriggerChars,
        provideCompletionItems: async (model, position, context) => {
          if (model.uri.toString() !== fileUri) {
            console.log(`[LSP-Completion] URI Mismatch: ${model.uri.toString()} !== ${fileUri}`);
            return { suggestions: [] };
          }

          const word = model.getWordUntilPosition(position);
          const range = {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: word.startColumn,
            endColumn: word.endColumn
          };

          console.log(`[LSP-Completion] Requesting for ${languageId} at ${position.lineNumber}:${position.column}, prefix: "${word.word}"`);
          
          try {
            const res: any = await sendRequest("textDocument/completion", {
              textDocument: { uri: fileUri },
              position: { line: position.lineNumber - 1, character: position.column - 1 },
              context: {
                triggerKind: context.triggerKind === monaco.languages.CompletionTriggerKind.TriggerCharacter ? 2 : 1,
                triggerCharacter: context.triggerCharacter
              }
            });
            
            if (!res) {
              console.log(`[LSP-Completion] No results from server`);
              return { suggestions: [] };
            }
            
            const items = Array.isArray(res) ? res : res.items;
            console.log(`[LSP-Completion] Got ${items?.length || 0} items`);
            if (items && items.length > 0) {
              console.log(`[LSP-Completion] First item sample:`, items[0]);
            }

            if (!items || items.length === 0) return { suggestions: [] };

            return {
              suggestions: items.map((item: any) => {
                // If the server provides a textEdit, use it for the range and text
                const textEdit = item.textEdit;
                const insertText = textEdit ? textEdit.newText : (item.insertText || item.label);
                
                let itemRange = range;
                if (textEdit && textEdit.range) {
                  itemRange = {
                    startLineNumber: textEdit.range.start.line + 1,
                    startColumn: textEdit.range.start.character + 1,
                    endLineNumber: textEdit.range.end.line + 1,
                    endColumn: textEdit.range.end.character + 1
                  };
                }

                return {
                  label: item.label,
                  kind: item.kind || monaco.languages.CompletionItemKind.Function,
                  insertText: insertText,
                  insertTextRules: item.insertTextFormat === 2 
                    ? monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet 
                    : monaco.languages.CompletionItemInsertTextRule.None,
                  detail: item.detail,
                  documentation: item.documentation,
                  range: itemRange,
                  sortText: item.sortText,
                  filterText: item.filterText
                };
              })
            };
          } catch (e) {
            console.error(`[LSP-Completion] Error:`, e);
            return { suggestions: [] };
          }
        }
      });

      // Crucial: cleanup providers when editor is destroyed
      editor.onDidDispose(() => {
        hoverProv.dispose();
        compProv.dispose();
      });

      console.log(`[LSP-Manual] Handshake success for ${languageId}`);
    };
  };

  return { initLsp };
};
