use crate::server::{JsonnetServer, LSPConnection, LSPServerManager};

mod bridge;
mod cache;
mod completion;
mod cst;
mod diagnostics;
mod node;
mod server;
mod utils;

pub mod binding {
    #![allow(warnings)]
    rust2go::r2g_include_binding!();
}

#[tokio::main]
async fn main() {
    let server = LSPServerManager {
        server: JsonnetServer {
            connection: LSPConnection::new_network(4874),
            ..Default::default()
        },
    };
    server.run().unwrap();
    //main_loop(server).unwrap()
}
