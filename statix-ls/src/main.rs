mod linter;
mod output;
mod settings;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server};

use settings::Settings;

struct StatixLsServer {
    client: Client,
    settings: Mutex<Settings>,
    stored_actions: Mutex<HashMap<Url, Vec<CodeAction>>>,
}

impl StatixLsServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            settings: Mutex::new(Settings::default()),
            stored_actions: Mutex::new(HashMap::new()),
        }
    }

    async fn lint_uri(&self, uri: Url) {
        let settings = self.settings.lock().await.clone();
        let (diagnostics, actions) = match linter::run(&uri, &settings).await {
            Ok(r) => r,
            Err(e) => {
                self.client
                    .log_message(MessageType::WARNING, format!("statix-ls: {e}"))
                    .await;
                return;
            }
        };

        self.stored_actions
            .lock()
            .await
            .insert(uri.clone(), actions);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for StatixLsServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(opts) = params.initialization_options {
            *self.settings.lock().await = serde_json::from_value(opts).unwrap_or_default();
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(false),
                        })),
                        ..Default::default()
                    },
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "statix-ls initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.lint_uri(params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.lint_uri(params.text_document.uri).await;
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        *self.settings.lock().await = serde_json::from_value(params.settings).unwrap_or_default();
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let request_range = params.range;

        let actions = self.stored_actions.lock().await;
        let file_actions = match actions.get(uri) {
            Some(a) => a,
            None => return Ok(None),
        };

        let response: Vec<CodeActionOrCommand> = file_actions
            .iter()
            .filter(|action| {
                action
                    .diagnostics
                    .as_ref()
                    .and_then(|d| d.first())
                    .is_some_and(|d| ranges_overlap(d.range, request_range))
            })
            .cloned()
            .map(CodeActionOrCommand::CodeAction)
            .collect();

        if response.is_empty() {
            Ok(None)
        } else {
            Ok(Some(response))
        }
    }
}

fn ranges_overlap(a: Range, b: Range) -> bool {
    a.start <= b.end && b.start <= a.end
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Arc::new(StatixLsServer::new(client)));
    Server::new(stdin, stdout, socket).serve(service).await;
}
