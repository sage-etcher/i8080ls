
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer};

use crate::data_types::FxDashMap;
use crate::parser::FileContext;

#[derive(Debug)]
pub struct Backend {
    client: Client,
    context: FxDashMap<Uri, FileContext>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            context: FxDashMap::default(),
        }
    }
}

impl Backend {
    fn get_diagnostics(&self, uri: Uri) -> Vec<Diagnostic> {
        dbg!("diagnostic");

        let ctx: &FileContext = &self.context.get(&uri).unwrap();

        let mut full_diagnostics: Vec<Diagnostic> = Vec::default();
        for error in &ctx.parser.error_list {
            full_diagnostics.push(Diagnostic {
                range: error.range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: String::from(error.get_errstr()),
                ..Default::default()
            });
        }

        dbg!(&ctx.parser.macro_list);

        return full_diagnostics;
    }

    async fn parse_file(&self, uri: Uri, text: &str) {
        // parse the file
        let mut ctx = FileContext::new(text);
        ctx.parse_file();
        self.context.insert(uri.clone(), ctx);

        // publish diagnostics
        self.client
            .publish_diagnostics(uri.clone(), self.get_diagnostics(uri), None)
            .await;
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> 
            Result<InitializeResult> {
        dbg!("initialize");

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: String::from("intel8080 LSP"),
                version: Some(String::from("0.1.0")),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        dbg!("initialized");
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        dbg!("shutdown");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        dbg!("did_open");
        let uri: Uri = params.text_document.uri;
        let text: &String = &params.text_document.text;

        self.parse_file(uri, text).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        dbg!("did_close");
        let uri = params.text_document.uri;
        self.context.remove(&uri);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        dbg!("did_change");
        let uri = params.text_document.uri;

        // initialize FileContext
        let change = &params.content_changes.first().unwrap();
        assert_eq!(change.range, None); // only support SyncKind::FULL

        // parse the file
        self.parse_file(uri, &change.text).await;
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        dbg!("hover");

        let hover_value: String = String::from("hover test");

        Ok(Some(Hover {
            range: None,
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_value,
            }),
        }))
    }
}
