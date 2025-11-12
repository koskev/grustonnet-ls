use grustonnet_ls_lib::server::config::{DiagnosticConfig, VariableNaming};

use crate::diagnostics::DiagnosticTestCase;

fn check_empty_diagnostics(val: &str) {
    DiagnosticTestCase {
        filename: val.to_string(),
        expected: vec![],
        config: DiagnosticConfig {
            enable_eval: true,
            enable_go_lint: true,
            unused_variables: true,
            variable_naming: VariableNaming::None,
            local_function: false,
            prevent_dollar: false,
            recursive_arguments: false,
            shadow_variable: false,
            duplicate_detection: false,
        },
    }
    .check();
}

test_macros::generate_test_function_for_dir!("testdata/complete/", check_empty_diagnostics);
test_macros::generate_test_function_for_dir!("testdata/definition/", check_empty_diagnostics);
