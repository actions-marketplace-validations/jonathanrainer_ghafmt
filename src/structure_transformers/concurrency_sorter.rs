//! Sorts keys under every `concurrency` mapping into idiomatic order.
use fyaml::Document;

use crate::structure_transformers::{StructureTransformer, for_each_mapping_child};

#[derive(Default)]
/// Sorts keys under `concurrency` into idiomatic order.
pub(crate) struct ConcurrencySorter;

/// Canonical key order within a `concurrency` mapping.
const CONCURRENCY_ORDERING: [&str; 2] = ["group", "cancel-in-progress"];

impl StructureTransformer for ConcurrencySorter {
    fn process(&self, mut doc: Document) -> fyaml::Result<Document> {
        let order = CONCURRENCY_ORDERING.map(String::from).to_vec();
        doc = self.sort_mapping_at_path(doc, "concurrency", &order)?;
        doc = for_each_mapping_child(doc, "jobs", |doc, job_path| {
            self.sort_mapping_at_path(doc, &format!("{job_path}/concurrency"), &order)
        })?;
        Ok(doc)
    }

    fn name(&self) -> &'static str {
        "concurrency-sorter"
    }

    fn description(&self) -> &'static str {
        "Sort keys under 'concurrency'"
    }
}

#[cfg(test)]
mod tests {
    use fyaml::Document;
    use indoc::indoc;
    use rstest::rstest;
    use similar_asserts::assert_eq;

    use super::*;

    #[rstest]
    #[case::no_concurrency_key_present(
        Document::from_string(indoc! {"
            a: b
            b: c
            d: z
            jobs: {}
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            a: b
            b: c
            d: z
            jobs: {}
        "}.to_string()
    )]
    #[case::concurrency_present(
        Document::from_string(indoc! {"
            a: b
            concurrency:
                cancel-in-progress: true
                group: foo
            b: c
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            a: b
            concurrency:
              group: foo
              cancel-in-progress: true
            b: c
        "}.to_string()
    )]
    #[case::job_concurrency_present(
        Document::from_string(indoc! {"
            jobs:
                bar: a
                baz: b
                foo:
                    concurrency:
                        cancel-in-progress: true
                        group: foo
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              bar: a
              baz: b
              foo:
                concurrency:
                  group: foo
                  cancel-in-progress: true
        "}.to_string()
    )]
    fn test_concurrency_sorter(#[case] source_doc: Document, #[case] expected: String) {
        let result = ConcurrencySorter
            .process(source_doc)
            .expect("processing failed")
            .to_string();

        assert_eq!(result, expected);
    }
}
