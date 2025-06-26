use std::{
    fs::{self},
    path::Path,
};

fn get_stdlib_urls(version: &str) -> Vec<(String, String)> {
    vec![
        (
            "stdlib-content.jsonnet".to_string(),
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
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    let urls = get_stdlib_urls("v0.21.0");
    for (name, url) in urls {
        let url_path = out_path.join(name);
        if !url_path.exists() {
            let content = reqwest::blocking::get(url).unwrap().text().unwrap();
            fs::write(url_path, content).unwrap();
        }
    }
}

fn main() {
    build_stdlib();
}
