use clap::Parser;
use env_logger::Env;
use grustonnet_config::Configuration;
use grustonnet_ls_lib::server::jsonnet::JsonnetServer;
use language_server::server::{LSPConnection, LSPServerManager};
use schemars::schema_for;

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
    if std::env::var("GODEBUG").is_err() {
        // At this point we are single threaded. Therefore this is safe
        unsafe {
            // Go seems to scan the stack an will panic upon encountering a 0x1 pointer.
            // However, Rust does use this value in some cases
            // If this turns out to be a problem we'll need to switch to an ipc based solution
            std::env::set_var("GODEBUG", "invalidptr=0,cgocheck=0");
        }

        let exe = std::env::current_exe().expect("Could not get current exe");
        let args = std::env::args();

        let err = exec::execvp(exe, args);

        eprintln!("Could not run execvp: {}", err);
        std::process::exit(1);
    }

    #[cfg(feature = "tracing")]
    tracy_client::Client::start();
    let args = Args::parse();

    if args.export_config_schema {
        println!(
            "{}",
            serde_json::to_string_pretty(&schema_for!(Configuration)).unwrap()
        );
        return;
    }
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
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
