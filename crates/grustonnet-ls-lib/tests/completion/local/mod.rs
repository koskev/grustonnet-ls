use grustonnet_ls_lib::server::config::CompletionConfig;
use grustonnet_ls_lib::server::config::Configuration;
use lsp_types::CompletionItem;
use lsp_types::CompletionList;

fn local_config() -> Configuration {
    Configuration {
        completion: CompletionConfig {
            enable_keywords: false,
            enable_global: false,
            enable_local: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

use crate::completion::common::CompletionTestCase;

pub mod array;
pub mod binary;
pub mod builder;
pub mod conditional;
pub mod dollar;
pub mod extcode;
pub mod function;
pub mod import;
pub mod locals;
pub mod object;
pub mod selfnode;
pub mod shadow;
pub mod stdlib;
pub mod superindex;
