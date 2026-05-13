
use tower_lsp_server::{LspService, Server};

use i8080ls::handler::Backend;


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
