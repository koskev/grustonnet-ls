use lsp_types::Position;

use super::*;

#[test]
fn var_single() {
    InlayHintTestCase {
        filename: "testdata/inlay_hints/apply/var_single.jsonnet".into(),
        range: Range {
            start: Position {
                line: 3,
                character: 0,
            },
            end: Position {
                line: 3,
                character: 15,
            },
        },
        hints: vec![InlayHint {
            label: InlayHintLabel::String("arg=".into()),
            padding_right: Some(true),
            position: Position {
                line: 3,
                character: 11,
            },

            ..default_inlay()
        }],
    }
    .check();
}

#[test]
fn var_multi() {
    InlayHintTestCase {
        filename: "testdata/inlay_hints/apply/var_multi.jsonnet".into(),
        range: Range {
            start: Position {
                line: 3,
                character: 0,
            },
            end: Position {
                line: 3,
                character: 20,
            },
        },
        hints: vec![
            InlayHint {
                label: InlayHintLabel::String("arg1=".into()),
                padding_right: Some(true),
                position: Position {
                    line: 3,
                    character: 11,
                },

                ..default_inlay()
            },
            InlayHint {
                label: InlayHintLabel::String("arg2=".into()),
                padding_right: Some(true),
                position: Position {
                    line: 3,
                    character: 14,
                },

                ..default_inlay()
            },
            InlayHint {
                label: InlayHintLabel::String("arg3=".into()),
                padding_right: Some(true),
                position: Position {
                    line: 3,
                    character: 17,
                },

                ..default_inlay()
            },
        ],
    }
    .check();
}

#[test]
fn var_multiline() {
    InlayHintTestCase {
        filename: "testdata/inlay_hints/apply/var_multiline.jsonnet".into(),
        range: Range {
            start: Position {
                line: 3,
                character: 0,
            },
            end: Position {
                line: 6,
                character: 4,
            },
        },
        hints: vec![
            InlayHint {
                label: InlayHintLabel::String("arg1=".into()),
                padding_right: Some(true),
                position: Position {
                    line: 4,
                    character: 4,
                },

                ..default_inlay()
            },
            InlayHint {
                label: InlayHintLabel::String("arg2=".into()),
                padding_right: Some(true),
                position: Position {
                    line: 5,
                    character: 4,
                },

                ..default_inlay()
            },
            InlayHint {
                label: InlayHintLabel::String("arg3=".into()),
                padding_right: Some(true),
                position: Position {
                    line: 6,
                    character: 4,
                },

                ..default_inlay()
            },
        ],
    }
    .check();
}

#[test]
fn index_single() {
    InlayHintTestCase {
        filename: "testdata/inlay_hints/apply/index_single.jsonnet".into(),
        range: Range {
            start: Position {
                line: 5,
                character: 0,
            },
            end: Position {
                line: 5,
                character: 25,
            },
        },
        hints: vec![InlayHint {
            label: InlayHintLabel::String("arg=".into()),
            padding_right: Some(true),
            position: Position {
                line: 5,
                character: 18,
            },

            ..default_inlay()
        }],
    }
    .check();
}

#[test]
fn index_multi() {
    InlayHintTestCase {
        filename: "testdata/inlay_hints/apply/index_multi.jsonnet".into(),
        range: Range {
            start: Position {
                line: 5,
                character: 0,
            },
            end: Position {
                line: 5,
                character: 25,
            },
        },
        hints: vec![
            InlayHint {
                label: InlayHintLabel::String("arg1=".into()),
                padding_right: Some(true),
                position: Position {
                    line: 5,
                    character: 18,
                },

                ..default_inlay()
            },
            InlayHint {
                label: InlayHintLabel::String("arg2=".into()),
                padding_right: Some(true),
                position: Position {
                    line: 5,
                    character: 21,
                },

                ..default_inlay()
            },
            InlayHint {
                label: InlayHintLabel::String("arg3=".into()),
                padding_right: Some(true),
                position: Position {
                    line: 5,
                    character: 24,
                },

                ..default_inlay()
            },
        ],
    }
    .check();
}
