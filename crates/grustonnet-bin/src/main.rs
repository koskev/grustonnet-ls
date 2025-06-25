use clap::Parser;
use grustonnet_ls_lib::server::{
    config::Configuration,
    jsonnet::JsonnetServer,
    server::{LSPConnection, LSPServerManager},
};
use schemars::schema_for;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    export_config_schema: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.export_config_schema {
        println!(
            "{}",
            serde_json::to_string_pretty(&schema_for!(Configuration)).unwrap()
        );
        return;
    }
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
