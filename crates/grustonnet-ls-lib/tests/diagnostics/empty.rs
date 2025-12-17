use grustonnet_config::{DiagnosticConfig, UnusedVariablesConfig};

use crate::diagnostics::{DiagnosticTestCase, IgnoreFields, disabled_diagnostics_config};

fn check_empty_diagnostics(val: &str) {
    DiagnosticTestCase {
        filename: val.to_string(),
        expected: vec![],
        config: DiagnosticConfig {
            enable_eval: true,
            unused_variables: UnusedVariablesConfig::default(),
            ..disabled_diagnostics_config()
        },
        ignore: IgnoreFields {
            source: true,
            ..Default::default()
        },
    }
    .check();
}

test_macros::generate_test_function_for_dir!("testdata/complete/", check_empty_diagnostics);
test_macros::generate_test_function_for_dir!("testdata/definition/", check_empty_diagnostics);
