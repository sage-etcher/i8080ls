
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer};

use crate::data_types::{FxDashMap, FxDashSet};
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

        // add parsing errors
        for error in &ctx.parser.error_list {
            full_diagnostics.push(Diagnostic {
                range: error.range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: String::from(error.get_errstr()),
                ..Default::default()
            });
        }

        // loop over macro_list, throw warnings for any label that does not
        // have any references
        let mut macro_iter = ctx.parser.macro_list.iter();
        loop {
            // loop 
            let macro_elem = macro_iter.next();
            if macro_elem.is_none() {
                break;
            }

            let macro_elem_unwrap = macro_elem.unwrap();
            let macro_value = macro_elem_unwrap.value();

            if macro_value.declaration.is_none() {
                let mut ref_iter = macro_value.references.iter();
                loop {
                    let ref_elem = ref_iter.next();
                    if ref_elem.is_none() {
                        break;
                    }

                    full_diagnostics.push(Diagnostic {
                        range: *ref_elem.unwrap().key(),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: String::from("undefined macro"),
                        ..Default::default()
                    });
                }
                continue;
            }

            if macro_value.references.len() == 0 {
                full_diagnostics.push(Diagnostic {
                    range: macro_value.declaration.unwrap(),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: String::from("unused macro"),
                    ..Default::default()
                });
            }
        }

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

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        dbg!("hover");
        let position = params.text_document_position_params;
        let uri = position.text_document.uri;
        let ctx: &FileContext = &self.context.get(&uri).unwrap();


        let mut text = ctx.parser.lexer.file_content.lines();
        let line_text: Vec<char> = text.nth(position.position.line as usize)
                                       .unwrap()
                                       .chars()
                                       .collect();

        let mut i = position.position.character as usize;

        // go to start of symbol
        while i > 0 {
            let ch = line_text[i-1];

            if !(ch.is_alphanumeric() || ch == '$' || ch == '_') {
                break;
            }

            i -= 1;
            dbg!(&i);
            dbg!(&ch);
        }

        // get ident
        let mut ident_vec: Vec<char> = Vec::default();
        while i < line_text.len() {
            let ch = line_text[i];
            if ch == '$' {
                i += 1;
                continue;
            }

            if !(ch.is_alphanumeric()|| ch == '_') {
                break;
            }

            ident_vec.push(ch);
            i += 1;
        }

        dbg!(&ident_vec);


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
