use std::{ffi::OsStr, fs};

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, parse::Parse, parse_macro_input, spanned::Spanned, token::Comma};
use walkdir::WalkDir;

struct MacroInput {
    dir: LitStr,
    _comma: Comma,
    func: Ident,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            dir: input.parse()?,
            _comma: input.parse()?,
            func: input.parse()?,
        })
    }
}

fn sanitize_identifier(val: &str) -> String {
    val.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Generates a function for every file in the given. Args are
/// path: Relative to the CARGO_MANIFEST_DIR
/// func: A function to call for every jsonnet file found
#[proc_macro]
pub fn generate_test_function_for_dir(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as MacroInput);
    let target_func = args.func;
    let root_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = format!("{}/{}", root_dir, args.dir.value());

    let path = fs::canonicalize(&dir).unwrap_or_else(|_| panic!("No abs path found for {dir}"));

    let walk = WalkDir::new(path);
    let funcs: Vec<_> = walk
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| {
            f.path().is_file()
                && f.path()
                    .extension()
                    .map(|e| e == "jsonnet")
                    .unwrap_or(false)
        })
        .map(|f| {
            let p = f.path();
            let func_name = sanitize_identifier(&format!(
                "test_{}_{}",
                p.parent()
                    .expect("No parent")
                    .file_name()
                    .unwrap_or(OsStr::new("unknown"))
                    .to_str()
                    .unwrap(),
                p.file_stem().expect("No stem").to_str().unwrap()
            ));
            let func_ident = Ident::new(&func_name, func_name.span());
            let path_str = p.to_str().unwrap();

            quote! {
                #[test]
                fn #func_ident() {
                    #target_func(#path_str);
                }
            }
        })
        .collect();

    let expanded = quote! {
        #(#funcs)*
    };

    TokenStream::from(expanded)
}
