use language_server::completion::Completion;
use lazy_static::lazy_static;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, CompletionTextEdit, InsertTextFormat,
    Position, Range, TextEdit,
};

pub struct DocsonnetSnippets {}

lazy_static! {
    static ref DOCSONNET_SNIPPETS: Vec<(&'static str, &'static str)> = vec![
        (
            "newvalue",
            r#"'#${1:Name}':: __.val(
    |||
       ${2: Help Text}
    |||,
    __.T.${3:any},
),
${1:Name}: ${4: Default value},
"#
        ),
        (
            "newfunction",
            r#"'#${1:Name}':: __.fn(
    |||
       ${2: Help Text}
    |||,
    [
    $0
    ]
),
${1:Name}(${3:Args}):: ${4: Default value},
"#
        ),
        (
            "newarg",
            r#"__.arg('${1:Name}', __.T.${2:any}, help='${3:Help}'),"#,
        )
    ];
}

impl Completion for DocsonnetSnippets {
    fn complete(
        &self,
        location: lsp_types::Position,
        _uri: &lsp_types::Uri,
    ) -> language_server::completion::CompletionResult {
        let start_location = Position {
            line: location.line,
            // Subtract one to be at the cursor and not one ahead
            character: location.character.checked_sub(1).unwrap_or_default(),
        };
        Ok(CompletionList {
            is_incomplete: false,
            items: DOCSONNET_SNIPPETS
                .iter()
                .map(|(name, val)| CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                        range: Range {
                            start: start_location,
                            end: location,
                        },
                        new_text: val.to_string(),
                    })),
                    ..Default::default()
                })
                .collect(),
        })
    }
}
