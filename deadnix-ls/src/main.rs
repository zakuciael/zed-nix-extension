mod linter;
mod output;
mod settings;

use std::sync::Arc;

use tokio::sync::Mutex;
use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server};

use settings::Settings;

struct DeadnixLsServer {
    client: Client,
    settings: Mutex<Settings>,
}

impl DeadnixLsServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            settings: Mutex::new(Settings::default()),
        }
    }

    async fn lint_uri(&self, uri: Url) {
        let settings = self.settings.lock().await.clone();
        let diagnostics = match linter::run(&uri, &settings).await {
            Ok(d) => d,
            Err(e) => {
                self.client
                    .log_message(MessageType::WARNING, format!("deadnix-ls: {e}"))
                    .await;
                return;
            }
        };

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for DeadnixLsServer {
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
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "deadnix-ls initialized")
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
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Arc::new(DeadnixLsServer::new(client)));
    Server::new(stdin, stdout, socket).serve(service).await;
}
