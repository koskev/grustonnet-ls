use super::*;

#[test]
fn builder_simple() {
    CompletionTestCase {
        filename: "testdata/complete/builder/simple.jsonnet".into(),
        replace_string: "x: self.new()".into(),
        replace_by_string: "x: self.new().".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "withArg".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withoutArg".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn builder_simple_without_arg() {
    CompletionTestCase {
        filename: "testdata/complete/builder/simple.jsonnet".into(),
        replace_string: "x: self.new()".into(),
        replace_by_string: "x: self.new().withoutArg().".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "noArg".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withArg".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withoutArg".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn builder_simple_without_arg_chain() {
    CompletionTestCase {
        filename: "testdata/complete/builder/simple.jsonnet".into(),
        replace_string: "x: self.new()".into(),
        replace_by_string: "x: self.new().withoutArg().withoutArg().withoutArg().withoutArg().withoutArg().".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "noArg".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withArg".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withoutArg".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn builder_simple_with_arg() {
    CompletionTestCase {
        filename: "testdata/complete/builder/simple.jsonnet".into(),
        replace_string: "x: self.new()".into(),
        replace_by_string: "x: self.new().withArg(1).".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "key".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withArg".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withoutArg".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
fn builder_simple_with_arg_chain() {
    CompletionTestCase {
        filename: "testdata/complete/builder/simple.jsonnet".into(),
        replace_string: "x: self.new()".into(),
        replace_by_string: "x: self.new().withArg(1).withArg(1).withArg(1).withArg(1).withArg(1)."
            .into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "key".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withArg".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withoutArg".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}


#[test]
#[ignore = "not implemented"]
fn builder_simple_mixed_arg_chain() {
    CompletionTestCase {
        filename: "testdata/complete/builder/simple.jsonnet".into(),
        replace_string: "x: self.new()".into(),
        replace_by_string: "x: self.new().withArg(1).withoutArg().withArg(1).withoutArg().withArg(1).withoutArg().withArg(1).withoutArg()."
            .into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![
                CompletionItem {
                    label: "key".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "noArg".to_string(),
                    ..Default::default()
                },
                
                CompletionItem {
                    label: "withArg".to_string(),
                    ..Default::default()
                },
                CompletionItem {
                    label: "withoutArg".to_string(),
                    ..Default::default()
                },
            ],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}

#[test]
#[ignore = "not implemented"]
fn builder_simple_with_arg_complete() {
    CompletionTestCase {
        filename: "testdata/complete/builder/simple.jsonnet".into(),
        replace_string: "x: self.new()".into(),
        replace_by_string: "x: self.new().withArg({inner: 5}).key.".into(),
        expected: CompletionList {
            is_incomplete: false,
            items: vec![CompletionItem {
                label: "inner".to_string(),
                ..Default::default()
            }],
        },
        config: local_config(),
        ..Default::default()
    }
    .check();
}
