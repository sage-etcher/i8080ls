
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer};

use crate::parser::{FileContext, TextPosition};
use crate::data_types::FxDashMap;


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

impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> 
            Result<InitializeResult> {
        dbg!("initialize");

        Ok(InitializeResult {
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        dbg!("did_open");
        let uri: Uri = params.text_document.uri;
        let text: &String = &params.text_document.text;

        // get text file contents
        let mut ctx: FileContext = FileContext::new(text.clone());
        ctx.parse_file();

        self.context.insert(uri, ctx);
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

        let file_content = change.text.split('\n').collect();
        let mut ctx: FileContext = FileContext::new(file_content);

        // Parse the file
        ctx.parse_file();

        // save the FileContext
        self.context.insert(uri, ctx);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        dbg!("hover");

        let uri: Uri = params.text_document_position_params.text_document.uri;
        let ctx: &FileContext = &self.context.get(&uri).unwrap();

        let comment: &TextPosition = &ctx.comments.get("debug_hover").unwrap();

        let hover_value: String = format!("# {}\n", comment.text);

        Ok(Some(Hover {
            range: None,
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_value,
            }),
        }))
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
}
