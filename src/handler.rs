
use std::collections::HashMap;
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
    // {{{
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
    
    fn get_ident(&self, uri: Uri, position: Position) -> (String, Range) {
        let ctx: &FileContext = &self.context.get(&uri).unwrap();

        let mut text = ctx.parser.lexer.file_content.lines();
        let line_text: Vec<char> = text.nth(position.line as usize)
                                       .unwrap()
                                       .chars()
                                       .collect();

        let mut i = position.character as usize;

        // go to start of symbol
        while i > 0 {
            let ch = line_text[i-1];

            if !(ch.is_alphanumeric() || ch == '$' || ch == '_') {
                break;
            }

            i -= 1;
        }
        let start_pos = Position {
            line:      position.line,
            character: i as u32,
        };

        // get ident
        let mut ident_vec: Vec<char> = Vec::default();
        while i < line_text.len() {
            let ch = line_text[i];
            if ch == '$' || ch == '_' {
                i += 1;
                continue;
            }

            if !ch.is_alphanumeric() {
                break;
            }

            ident_vec.push(ch);
            i += 1;
        }
        let end_pos = Position {
            line:      position.line,
            character: i as u32,
        };

        let range = Range {
            start: start_pos,
            end:   end_pos,
        };

        let ident: String = ident_vec.into_iter().collect();
        let ident_lower: String = ident.to_lowercase().to_string();

        return (ident_lower, range);
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
    // }}}
}

impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> 
            Result<InitializeResult> {
        dbg!("initialize");
        dbg!(&_params);

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
                // completion_provider: Some(CompletionOptions {
                //
                // }),
                definition_provider:         Some(OneOf::Left(true)),
                references_provider:         Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: Some(false),
                    },
                })),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: WorkDoneProgressOptions {
                                work_done_progress: Some(false),
                            },
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,   // OPCODE
                                    SemanticTokenType::MACRO,     // MACRO
                                    SemanticTokenType::PARAMETER, // REGISTER
                                    SemanticTokenType::VARIABLE,  // LABEL
                                    SemanticTokenType::NUMBER,    // NUMBER
                                    SemanticTokenType::STRING,    // STRING
                                    SemanticTokenType::COMMENT,   // COMMENT
                                    SemanticTokenType::OPERATOR,  // SYMBOL
                                    SemanticTokenType::DECORATOR, // SPACERS
                                    SemanticTokenType::TYPE,      // NUMTYPE
                                ],
                                token_modifiers: vec![],
                            },
                            range: None,
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        }
                    )
                ),

                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        // {{{
        dbg!("initialized");
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
        // }}}
    }

    async fn shutdown(&self) -> Result<()> {
        // {{{
        dbg!("shutdown");
        Ok(())
        // }}}
    }

    // buffer updates
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        // {{{
        dbg!("did_open");
        let uri: Uri = params.text_document.uri;
        let text: &String = &params.text_document.text;

        self.parse_file(uri, text).await;
        // }}}
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // {{{
        dbg!("did_close");
        let uri = params.text_document.uri;
        self.context.remove(&uri);
        // }}}
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // {{{
        dbg!("did_change");
        let uri = params.text_document.uri;

        // initialize FileContext
        let change = &params.content_changes.first().unwrap();
        assert_eq!(change.range, None); // only support SyncKind::FULL

        // parse the file
        self.parse_file(uri, &change.text).await;
        // }}}
    }

    // actions
    async fn goto_definition(&self, params: GotoDefinitionParams) -> 
                Result<Option<GotoDefinitionResponse>> {
        // {{{
        dbg!("goto_definition");
        let uri: Uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let ctx: &FileContext = &self.context.get(&uri).unwrap();

        let (ident, _) = self.get_ident(uri.clone(), position);
        let macro_match = ctx.parser.macro_list.get(&ident);

        if macro_match.is_none() {
            return Ok(None);
        }

        let macro_unwrap = macro_match.unwrap();
        if macro_unwrap.declaration.is_none() {
            return Ok(None);
        }

        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri,
            range: macro_unwrap.declaration.unwrap(),
        })))
        // }}}
    }

    async fn references(&self, params: ReferenceParams) -> 
                Result<Option<Vec<Location>>> {
        // {{{
        dbg!("references");
        let uri: Uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let ctx: &FileContext = &self.context.get(&uri).unwrap();

        let (ident, _) = self.get_ident(uri.clone(), position);
        let macro_match = ctx.parser.macro_list.get(&ident);

        if macro_match.is_none() {
            return Ok(None);
        }

        let macro_unwrap = macro_match.unwrap();
        if macro_unwrap.references.len() == 0 {
            return Ok(None);
        }

        let mut references_vec: Vec<Location> = Vec::new();
        let mut ref_iter = macro_unwrap.references.iter();
        loop {
            let ref_elem = ref_iter.next();
            if ref_elem.is_none() {
                break;
            }

            references_vec.push(Location {
                uri: uri.clone(),
                range: *ref_elem.unwrap(),
            });
        }

        Ok(Some(references_vec))
        // }}}
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // {{{
        dbg!("hover");
        let position = params.text_document_position_params.position;
        let uri: Uri = params.text_document_position_params.text_document.uri;
        let ctx: &FileContext = &self.context.get(&uri).unwrap();

        let (ident, _) = self.get_ident(uri, position);
        let macro_match = ctx.parser.macro_list.get(&ident);

        if macro_match.is_none() {
            return Ok(None)
        }

        let macro_unwrap = macro_match.unwrap();

        if macro_unwrap.value.is_none() {
            return Ok(None)
        }

        let mut hover_value = format!("# {}: {}", 
                                      macro_unwrap.key.clone(),
                                      macro_unwrap.value.clone().unwrap());

        if macro_unwrap.description.is_some() {
            if macro_unwrap.description.clone().unwrap().len() > 0 {
                hover_value = format!("{}\n{}", hover_value, 
                                      macro_unwrap.description.clone().unwrap());
            }
        }

        Ok(Some(Hover {
            range: None,
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_value,
            }),
        }))
        // }}}
    }

    async fn prepare_rename(&self, params: TextDocumentPositionParams) -> 
                Result<Option<PrepareRenameResponse>> {
        // {{{
        dbg!("prepare_rename");

        let position = params.position;
        let uri: Uri = params.text_document.uri;
        let ctx: &FileContext = &self.context.get(&uri).unwrap();

        let (ident, range) = self.get_ident(uri, position);
        if !ctx.parser.macro_list.contains_key(&ident) {
            return Ok(None);
        }

        Ok(Some(PrepareRenameResponse::Range(range)))
        // }}}
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        // {{{
        let position = params.text_document_position.position;
        let uri: Uri = params.text_document_position.text_document.uri;
        let ctx: &FileContext = &self.context.get(&uri).unwrap();

        let (ident, _) = self.get_ident(uri.clone(), position);
        let macro_get = ctx.parser.macro_list.get(&ident);
        if macro_get.is_none() {
            return Ok(None);
        }

        let macro_match = macro_get.unwrap();

        let mut change_list: Vec<TextEdit> = Vec::new();
        if !macro_match.declaration.is_none() {
            change_list.push(TextEdit {
                range:    macro_match.declaration.unwrap(),
                new_text: params.new_name.clone(),
            });
        }

        let mut ref_iter = macro_match.references.iter();
        loop {
            let ref_elem = ref_iter.next();
            if ref_elem.is_none() {
                break;
            }

            let range = *ref_elem.unwrap();
            dbg!(&range);

            change_list.push(TextEdit {
                range,
                new_text: params.new_name.clone(),
            });
        }

        let mut change_map: HashMap<Uri, Vec<TextEdit>> = HashMap::new();
        change_map.insert(uri.clone(), change_list);

        Ok(Some(WorkspaceEdit {
            changes: Some(change_map),
            document_changes: None,
            change_annotations: None,
        }))
        // }}}
    }

    // colors
    async fn semantic_tokens_full(&self, params: SemanticTokensParams) ->
                Result<Option<SemanticTokensResult>> {
        dbg!("semantic_tokesn_full");
        // OH THIS IS HARD TO TEST
        // I'm semi-stuck on neovim v0.8... see below lol
        //
        // # neovim semantic token support by version
        // | version | support |
        // |:------- |:------- |
        // | v0.8    | false   |
        // | v0.11   | true    |

        let uri: Uri = params.text_document.uri;
        let ctx: &FileContext = &self.context.get(&uri).unwrap();

        let mut semantic_token_data: Vec<SemanticToken> = Vec::default();
        let mut rel_position = Position {
            line: 0,
            character: 0,
        };

        for elem in &ctx.parser.semantic_list {
            semantic_token_data.push(SemanticToken {
                delta_line: elem.range.start.line - rel_position.line,
                delta_start: elem.range.start.character - rel_position.character,
                length: elem.range.end.character - elem.range.start.character + 1,
                token_type: elem.element_type as u32,
                token_modifiers_bitset: 0,
            });

            rel_position = elem.range.start;
        }

        match semantic_token_data.len() {
            0 => return Ok(None),
            _ => {
                return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: semantic_token_data,
                })));
            }
        }
    }
    
    //async fn document_symbol(&self, params: DocumentSymbolParams) ->
    //            Result<Option<DocumentSymbolResponse>> {
    //    // {{{
    //    dbg!("document_symbol");

    //    let uri: Uri = params.text_document.uri;
    //    let ctx: &FileContext = &self.context.get(&uri).unwrap();

    //    dbg!(&ctx.parser.semantic_list);

    //    Ok(None)
    //    // }}}
    //}
}

// vim: fdm=marker
