
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer};

use crate::code_elements::CodeMacro;
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
        let mut ctx: FileContext = FileContext::new(text);
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

        let mut ctx = FileContext::new(&change.text);
        ctx.parse_file();

        // save the FileContext
        self.context.insert(uri, ctx);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        dbg!("hover");

        let uri: Uri = params.text_document_position_params.text_document.uri;
        let ctx: &FileContext = &self.context.get(&uri).unwrap();

        let debug_hover: &CodeMacro = &ctx.macros.get("debug_hover").unwrap();
        let hover_value: String = format!("# {}\n", debug_hover.macro_text);

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
