use std::{
    fs::{self},
    path::Path,
};

use jsonnet_bridge::go::{ASTBridge, ASTBridgeImpl, EvaluateParams};

const STDLIB_FILE: &'static str = "stdlib-content.jsonnet";

fn get_stdlib_urls(version: &str) -> Vec<(String, String)> {
    vec![
        (
            STDLIB_FILE.to_string(),
            format!(
                "https://raw.githubusercontent.com/google/jsonnet/{}/doc/_stdlib_gen/stdlib-content.jsonnet",
                version
            ),
        ),
        (
            "html.libsonnet".to_string(),
            format!(
                "https://raw.githubusercontent.com/google/jsonnet/{}/doc/_stdlib_gen/html.libsonnet",
                version
            ),
        ),
    ]
}

fn build_stdlib() {
    // Use gen dir to avoid downloading the file again after each change
    let gen_path = Path::new("./gen");
    let _ = fs::create_dir(gen_path);
    let urls = get_stdlib_urls("v0.21.0");
    for (name, url) in urls {
        let url_path = gen_path.join(name);
        if !url_path.exists() {
            let content = reqwest::blocking::get(url).unwrap().text().unwrap();
            fs::write(url_path, content).unwrap();
        }
    }
    let content = include_str!("stdlib.jsonnet");
    let info = ASTBridgeImpl::evaluate_snippet(
        STDLIB_FILE.to_string(),
        content.to_string(),
        EvaluateParams {
            jpaths: vec![gen_path.to_str().unwrap().to_string()],
            ..Default::default()
        },
    );
    assert!(
        info.error_data.len() == 0,
        "Got eval error {:?}",
        info.error_data
    );
    assert!(info.ast_data.len() != 0, "No eval data {:?}", info.ast_data);
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    let stdlib_path = out_path.join("stdlib.json");
    fs::write(stdlib_path.clone(), info.ast_data).expect(&format!(
        "Failed to write stdlib to out path at {:?}",
        stdlib_path
    ));
}

fn main() {
    build_stdlib();
}
