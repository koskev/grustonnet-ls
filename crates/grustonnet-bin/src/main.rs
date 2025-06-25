use grustonnet_ls_lib::server::{JsonnetServer, LSPConnection, LSPServerManager};

#[tokio::main]
async fn main() {
    env_logger::init();
    let server = LSPServerManager {
        server: JsonnetServer {
            connection: LSPConnection::new_network(4874),
            ..Default::default()
        },
    };
    server.run().unwrap();
    //main_loop(server).unwrap()
}
