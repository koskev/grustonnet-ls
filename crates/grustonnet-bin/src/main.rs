use clap::Parser;
use env_logger::Env;
use grustonnet_config::Configuration;
use grustonnet_ls_lib::server::jsonnet::JsonnetServer;
use language_server::server::{LSPConnection, LSPServerManager};
use rust2go_env::restart_with_fixed_env;
use schemars::{generate::SchemaSettings, schema_for};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    export_config_schema: bool,

    #[arg(long, short)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    restart_with_fixed_env();

    #[cfg(feature = "tracing")]
    tracy_client::Client::start();
    let args = Args::parse();

    if args.export_config_schema {
        let settings = SchemaSettings::draft07().with(|s| {
            s.meta_schema = None;
            s.inline_subschemas = true;
        });
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<Configuration>();
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        return;
    }
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let connection = if let Some(port) = args.port {
        LSPConnection::new_network(port)
    } else {
        LSPConnection::default()
    };
    let server = LSPServerManager {
        server: JsonnetServer::new(connection),
    };
    server.run().unwrap();
}
