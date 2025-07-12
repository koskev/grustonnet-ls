use std::{
    fs::{self},
    path::Path,
};

use jsonnet_bridge::go::{ASTBridge, ASTBridgeImpl, EvaluateParams};
use jsonnet_std_docs::StdLib;

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
    let root_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let gen_dir = format!("{root_dir}/gen");
    let gen_path = Path::new(&gen_dir);
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

    // Convert html to md
    let mut lib: StdLib = serde_json::from_str(&info.ast_data).unwrap();
    lib.groups.iter_mut().for_each(|group| {
        group.fields.iter_mut().for_each(|func| {
            func.description = htmd::HtmlToMarkdown::new()
                .convert(&func.description)
                .unwrap();
        });
    });

    let out_content = serde_json::to_string(&lib).unwrap();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    let stdlib_path = out_path.join("stdlib.json");
    fs::write(stdlib_path.clone(), out_content).expect(&format!(
        "Failed to write stdlib to out path at {:?}",
        stdlib_path
    ));
}

fn main() {
    build_stdlib();
}
