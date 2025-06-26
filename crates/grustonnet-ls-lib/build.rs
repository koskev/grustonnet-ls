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
    let out_dir = "./gen";
    let out_path = Path::new(&out_dir);
    let _ = fs::create_dir(out_path);
    let urls = get_stdlib_urls("v0.21.0");
    for (name, url) in urls {
        let url_path = out_path.join(name);
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
            jpaths: vec![out_path.to_str().unwrap().to_string()],
            ..Default::default()
        },
    );
    assert!(
        info.error_data.len() == 0,
        "Got eval error {:?}",
        info.error_data
    );
    assert!(info.ast_data.len() != 0, "No eval data {:?}", info.ast_data);
    fs::write(out_path.join("stdlib.json"), info.ast_data).unwrap();
}

fn main() {
    build_stdlib();
}
