use grustonnet_ls_lib::server::config::DiagnosticConfig;

use crate::diagnostics::DiagnosticTestCase;

fn check_empty_diagnostics(val: &str) {
    DiagnosticTestCase {
        filename: val.to_string(),
        expected: vec![],
        config: DiagnosticConfig {
            enable_eval: true,
            enable_go_lint: true,
            enable_lint: true,
        },
    }
    .check();
}

test_macros::generate_test_function_for_dir!("testdata/complete/", check_empty_diagnostics);
test_macros::generate_test_function_for_dir!("testdata/definition/", check_empty_diagnostics);
