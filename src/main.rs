
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

use dashmap::DashMap;
use std::hash::BuildHasherDefault;
use rustc_hash::FxHasher;

pub type FastDashMap<K, V> = DashMap<K, V, BuildHasherDefault<FxHasher>>;

pub struct FileVersion(pub i32);

#[derive(Debug)]
struct TextPosition {
    text: String,
    _position: Range,

    _offset:  Option<u16>,
    _i_sbyte: Option<i8>,
    _i_byte:  Option<u8>,
    _i_word:  Option<u16>,
}

#[derive(Debug)]
struct FileContext {
    file_content: String,
    _macros:   FastDashMap<String, TextPosition>,
    _opcodes:  FastDashMap<String, TextPosition>,
    _strings:  FastDashMap<String, TextPosition>,
    _numbers:  FastDashMap<String, TextPosition>,
    comments: FastDashMap<String, TextPosition>,
    _symbols:  FastDashMap<String, TextPosition>,
}

#[derive(Debug)]
struct Backend {
    client: Client,
    context: FastDashMap<Uri, FileContext>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            context: FastDashMap::default(),
        }
    }
}

impl FileContext {
    pub fn new(file_content: String) -> Self {
        Self {
            file_content,
            _macros:   FastDashMap::default(),
            _opcodes:  FastDashMap::default(),
            _strings:  FastDashMap::default(),
            _numbers:  FastDashMap::default(),
            comments: FastDashMap::default(),
            _symbols:  FastDashMap::default(),
        }
    }

    pub fn parse_file(&mut self) {
        // parse stuff

        // Debuging comment
        self.comments.insert("debug_hover".to_string(), TextPosition {
            text: "hello this is a hover default".to_string(),
            _position: Range {
                start: Position {
                    line:      0,
                    character: 0,
                },
                end: Position {
                    line:      0,
                    character: 0,
                },
            },
            _offset:  None,
            _i_sbyte: None,
            _i_byte:  None,
            _i_word:  None,
        });
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
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

#[tokio::main]
async fn main() {
    let address = "127.0.0.1:9292";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("listening on {}", address);

    let (stream, _) = listener.accept().await.unwrap();
    let (read, write) = tokio::io::split(stream);

    let (service, socket) = LspService::new(Backend::new);

    Server::new(read, write, socket).serve(service).await;
}

// end of file
