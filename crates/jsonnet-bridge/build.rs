use std::{env, path::Path, process::Command};

use rust2go::{GoCompiler, LinkType};

fn main() {
    rust2go::Builder::new()
        .with_go_compiler(CrossGoCompiler {})
        .with_go_src("./go")
        .build();
}

#[derive(Debug, Clone, Copy)]
pub struct CrossGoCompiler;

impl GoCompiler for CrossGoCompiler {
    fn go_build(&self, go_src: &Path, link: LinkType, output: &Path) {
        let mut go_build = Command::new("go");

        // XXX: There is no environment variable to get the current CC and rust2go does not
        // automatically set the correct variables
        // cfg(target_os == "windows") does not work either
        if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
            go_build
                .env("CC", "x86_64-w64-mingw32-gcc")
                .env("CGO_ENABLED", "1")
                .env("GOOS", "windows")
                .env("GOARCH", "amd64");
        }

        go_build
            .env("GO111MODULE", "on")
            .current_dir(go_src)
            .arg("build")
            .arg(if link == LinkType::Static {
                "-buildmode=c-archive"
            } else {
                "-buildmode=c-shared"
            })
            .arg("-o")
            .arg(output)
            .arg(".");

        go_build.status().expect("Go build failed");
    }
}
