use std::path::PathBuf;

use clap::Parser;
use env_logger::Env;
use grustonnet_ls_lib::{
    diagnostics::filter::JsonnetDiagnosticFilter, server::jsonnet::JsonnetServer,
};
use language_server::diagnostics::DiagnosticFilter;
use language_server::utils::{UriHelper, rope::RopeHelper};
use lsp_types::{DiagnosticSeverity, Uri};
use miette::{LabeledSpan, miette};
use ropey::Rope;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,

    #[arg(long, short)]
    jpaths: Vec<String>,
}

trait SeverityMap {
    fn to_miette(&self) -> miette::Severity;
}

impl SeverityMap for DiagnosticSeverity {
    fn to_miette(&self) -> miette::Severity {
        match *self {
            DiagnosticSeverity::WARNING => miette::Severity::Warning,
            DiagnosticSeverity::ERROR => miette::Severity::Error,
            DiagnosticSeverity::INFORMATION | DiagnosticSeverity::HINT => miette::Severity::Advice,
            _ => miette::Severity::default(),
        }
    }
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

    env_logger::Builder::from_env(Env::default().default_filter_or("fatal")).init();

    let paths: Vec<PathBuf> = if args.path.is_dir() {
        glob::glob(&format!("{}/**/*.*sonnet", args.path.to_str().unwrap()))
            .unwrap()
            .filter_map(|g| {
                if g.as_ref().ok()?.is_file() {
                    Some(g.ok()?)
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![args.path.clone()]
    };
    let server = JsonnetServer::default();
    server
        .configuration
        .write()
        .unwrap()
        .jsonnet
        .jpaths
        .extend(args.jpaths);
    // TODO: this needs to go (how many TODOs do I have for this cursed config?)
    server
        .cache
        .ast_generator
        .jsonnet
        .set_config(&server.configuration.read().unwrap().jsonnet);
    let filter = JsonnetDiagnosticFilter::new(server.cache.clone());
    for path in &paths {
        let diags = server.get_diagnostics(&Uri::from_path(path).unwrap());
        let diags = filter.filter_diagnostics(&Uri::from_path(path).unwrap(), diags);
        let content = std::fs::read_to_string(path).unwrap();
        if !diags.is_empty() {
            eprintln!("Lint results for {:?}", path);
        }
        for diag in &diags {
            let source = content.clone();
            let rope = Rope::from_str(&source);
            let start = rope.get_index(diag.diagnostics.range.start);
            let end = rope.get_index(diag.diagnostics.range.end);
            let report = miette!(
                labels = vec![LabeledSpan::at(
                    start..end,
                    diag.diagnostics.message.clone()
                ),],
                severity = diag
                    .diagnostics
                    .severity
                    .unwrap_or(DiagnosticSeverity::ERROR)
                    .to_miette(),
                "Linter result"
            )
            .with_source_code(source);
            eprintln!("{:?}", report)
        }
    }
}
