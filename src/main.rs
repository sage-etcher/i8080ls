
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

use dashmap::DashMap;
use std::hash::BuildHasherDefault;
use rustc_hash::FxHasher;

pub type FastDashMap<K, V> = DashMap<K, V, BuildHasherDefault<FxHasher>>;

pub struct FileVersion(pub i32);

#[derive(Debug)]
struct BackendFileContext {
    my_map: FastDashMap<u32, u32>,
}

#[derive(Debug)]
struct Backend {
    client: Client,
    context: BackendFileContext,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            context: BackendFileContext {
                my_map: FastDashMap::default()
            },
        }
    }
}

impl BackendFileContext {
    pub fn insert_pair(&self, key: u32, value: u32) {
        self.my_map.insert(key, value);
    }

    pub fn remove_pair(&self, key: u32) {
        self.my_map.remove(&key);
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
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn did_change(&self, _params: DidChangeTextDocumentParams) {
        dbg!("did_change");

        self.context.insert_pair(10, 20);
        dbg!(&self.context);
        self.context.remove_pair(10);
        dbg!(&self.context);
        let uri = params.text_document.uri;
        //let mut file_contents: Option<String> = None;

        //for change in params.content_changes {
        //    file_contents = Some(change.text.to_string());
        //}

        //&self.file_context.insert(uri.clone(), BackendFileContext {
        //    uri,
        //    file_contents,
        //});

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
