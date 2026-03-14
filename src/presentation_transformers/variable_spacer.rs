//! Normalizes spacing inside `${{ ... }}` expressions in the emitted event stream.
use std::sync::LazyLock;

use fyaml::{EmitEvent, WriteType};
use regex::Regex;

use crate::presentation_transformers::PresentationTransformer;

/// Pre-compiled regex that matches a `${{ ... }}` expression, capturing the inner content.
#[allow(clippy::unwrap_used)] // literal pattern — always a valid regex
static VARIABLE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\$\{\{\s*(.*?)\s*}}").unwrap());

/// Ensures every `${{ expr }}` expression has exactly one space inside each brace pair.
#[derive(Default)]
pub(crate) struct VariableSpacer;

impl PresentationTransformer for VariableSpacer {
    fn process(&self, event_stream: Vec<EmitEvent>) -> Vec<EmitEvent> {
        // Scan over the event stream, and any time you find any kind of scalar
        // run the regex over it to ensure it's consistently spaced
        event_stream
            .into_iter()
            .map(
                |EmitEvent {
                     write_type,
                     content,
                 }| match write_type {
                    WriteType::PlainScalar
                    | WriteType::DoubleQuotedScalar
                    | WriteType::SingleQuotedScalar
                    | WriteType::LiteralScalar
                    | WriteType::FoldedScalar => {
                        let replaced = VARIABLE_RE.replace_all(&content, "$${{ $1 }}").into_owned();
                        EmitEvent {
                            write_type,
                            content: replaced,
                        }
                    }
                    _ => EmitEvent {
                        write_type,
                        content,
                    },
                },
            )
            .collect()
    }

    fn description(&self) -> &'static str {
        "Ensure correct spacing around variables"
    }
}

#[cfg(test)]
mod tests {
    use fyaml::{EmitEvent, WriteType};
    use rstest::rstest;
    use similar_asserts::assert_eq;

    use super::*;

    fn ev(write_type: WriteType, content: &str) -> EmitEvent {
        EmitEvent {
            write_type,
            content: content.to_string(),
        }
    }

    fn run(events: Vec<EmitEvent>) -> Vec<EmitEvent> {
        VariableSpacer.process(events)
    }

    #[rstest]
    #[case::already_spaced(
        vec![ev(WriteType::PlainScalar, "${{ github.ref }}")],
        vec![ev(WriteType::PlainScalar, "${{ github.ref }}")]
    )]
    #[case::no_spaces(
        vec![ev(WriteType::PlainScalar, "${{github.ref}}")],
        vec![ev(WriteType::PlainScalar, "${{ github.ref }}")]
    )]
    #[case::extra_spaces(
        vec![ev(WriteType::PlainScalar, "${{  github.ref  }}")],
        vec![ev(WriteType::PlainScalar, "${{ github.ref }}")]
    )]
    #[case::mixed_spacing(
        vec![ev(WriteType::PlainScalar, "${{github.ref }}")],
        vec![ev(WriteType::PlainScalar, "${{ github.ref }}")]
    )]
    #[case::multiple_variables(
        vec![ev(WriteType::PlainScalar, "${{a}}-${{b}}")],
        vec![ev(WriteType::PlainScalar, "${{ a }}-${{ b }}")]
    )]
    #[case::double_quoted_scalar(
        vec![ev(WriteType::DoubleQuotedScalar, "${{github.ref}}")],
        vec![ev(WriteType::DoubleQuotedScalar, "${{ github.ref }}")]
    )]
    #[case::single_quoted_scalar(
        vec![ev(WriteType::SingleQuotedScalar, "${{github.ref}}")],
        vec![ev(WriteType::SingleQuotedScalar, "${{ github.ref }}")]
    )]
    #[case::literal_scalar(
        vec![ev(WriteType::LiteralScalar, "${{github.ref}}")],
        vec![ev(WriteType::LiteralScalar, "${{ github.ref }}")]
    )]
    #[case::folded_scalar(
        vec![ev(WriteType::FoldedScalar, "${{github.ref}}")],
        vec![ev(WriteType::FoldedScalar, "${{ github.ref }}")]
    )]
    #[case::non_scalar_untouched(
        vec![ev(WriteType::Comment, "# ${{github.ref}}")],
        vec![ev(WriteType::Comment, "# ${{github.ref}}")]
    )]
    #[case::indicator_untouched(
        vec![ev(WriteType::Indicator, ":")],
        vec![ev(WriteType::Indicator, ":")]
    )]
    #[case::no_variable(
        vec![ev(WriteType::PlainScalar, "just plain text")],
        vec![ev(WriteType::PlainScalar, "just plain text")]
    )]
    #[case::expression_with_function(
        vec![ev(WriteType::PlainScalar, "${{contains(github.ref, 'main')}}")],
        vec![ev(WriteType::PlainScalar, "${{ contains(github.ref, 'main') }}")]
    )]
    fn test_variable_spacer(#[case] input: Vec<EmitEvent>, #[case] expected: Vec<EmitEvent>) {
        assert_eq!(run(input), expected);
    }
}
