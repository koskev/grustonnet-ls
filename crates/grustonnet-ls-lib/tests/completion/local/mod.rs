use grustonnet_config::CompletionConfig;
use grustonnet_config::Configuration;
use grustonnet_config::SnippetConfig;
use lsp_types::CompletionItem;
use lsp_types::CompletionList;

fn local_config() -> Configuration {
    Configuration {
        completion: CompletionConfig {
            enable_keywords: false,
            enable_global: false,
            enable_local: true,
            snippets: SnippetConfig { docsonnet: false },
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
pub mod documentation;
pub mod dollar;
pub mod errors;
pub mod extcode;
pub mod function;
pub mod import;
pub mod index;
pub mod locals;
pub mod object;
pub mod selfnode;
pub mod shadow;
pub mod std_completion;
pub mod stdlib;
pub mod superindex;
