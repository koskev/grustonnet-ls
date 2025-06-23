use crate::server::{JsonnetServer, LSPServerManager};

mod bridge;
mod cache;
mod completion;
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
        server: JsonnetServer::new(),
    };
    server.run().unwrap();
    //main_loop(server).unwrap()
}
