//! Sorts `with` entries in each step into idiomatic order.
use std::cmp::Ordering;

use fyaml::Document;

use crate::structure_transformers::{
    StructureTransformer, for_each_mapping_child, for_each_seq_element,
};

#[derive(Default)]
/// Sorts `with` entries in each step alphabetically.
pub(crate) struct WithSorter;

impl StructureTransformer for WithSorter {
    #[allow(clippy::match_same_arms, clippy::unwrap_used)]
    // scalar_str() returns Err if the node is not a scalar or contains non-UTF-8 bytes;
    // both are invariants that hold for any well-formed GHA workflow.
    fn process(&self, mut doc: Document) -> fyaml::Result<Document> {
        doc = for_each_mapping_child(doc, "/jobs", |doc, job_path| {
            for_each_seq_element(doc, &format!("{job_path}/steps"), |mut doc, step_path| {
                let with_path = format!("{step_path}/with");
                if doc.at_path(&with_path).is_some() {
                    doc.edit().sort_mapping_at(&with_path, |k1, _, k2, _| {
                        let key1 = k1.scalar_str().unwrap();
                        let key2 = k2.scalar_str().unwrap();

                        match (key1, key2) {
                            // Entrypoint and args are a special case: always entrypoint first,
                            // followed by args, and then everything else alphabetically.
                            // N.B. The first two arms are intentional: they establish the relative
                            // ordering between entrypoint and args before the general rules fire.
                            ("args", "entrypoint") => Ordering::Greater,
                            ("entrypoint", "args") => Ordering::Less,
                            (_, "entrypoint" | "args") => Ordering::Greater,
                            ("args" | "entrypoint", _) => Ordering::Less,
                            (_, _) => key1.cmp(key2),
                        }
                    })?;
                }
                Ok(doc)
            })
        })?;

        Ok(doc)
    }

    fn name(&self) -> &'static str {
        "with-sorter"
    }

    fn description(&self) -> &'static str {
        "Sort 'with' entries in a step in an idiomatic way"
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
    #[case::no_with(
        Document::from_string(indoc! {"
            jobs:
                build:
                    steps:
                        - run: echo hi
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                steps:
                - run: echo hi
        "}.to_string()
    )]
    #[case::entrypoint_args_and_extras_sorted(
        Document::from_string(indoc! {"
            jobs:
                build:
                    steps:
                        - uses: docker://alpine:3
                          with:
                              args: --verbose
                              config: /etc/app.conf
                              entrypoint: /bin/sh
                              debug: true
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                steps:
                - uses: docker://alpine:3
                  with:
                    entrypoint: /bin/sh
                    args: --verbose
                    config: /etc/app.conf
                    debug: true
        "}.to_string()
    )]
    #[case::entrypoint_with_extras(
        Document::from_string(indoc! {"
            jobs:
                build:
                    steps:
                        - uses: docker://python:3
                          with:
                              verbose: true
                              entrypoint: /usr/local/bin/python
                              script: main.py
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                steps:
                - uses: docker://python:3
                  with:
                    entrypoint: /usr/local/bin/python
                    script: main.py
                    verbose: true
        "}.to_string()
    )]
    #[case::args_with_extras(
        Document::from_string(indoc! {"
            jobs:
                build:
                    steps:
                        - uses: docker://ruby:3
                          with:
                              timeout: '30'
                              args: --strict
                              retries: '3'
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                steps:
                - uses: docker://ruby:3
                  with:
                    args: --strict
                    retries: '3'
                    timeout: '30'
        "}.to_string()
    )]
    #[case::extras_only_alphabetical(
        Document::from_string(indoc! {"
            jobs:
                build:
                    steps:
                        - uses: actions/checkout@v4
                          with:
                              fetch-depth: 0
                              clean: true
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                steps:
                - uses: actions/checkout@v4
                  with:
                    clean: true
                    fetch-depth: 0
        "}.to_string()
    )]
    #[case::already_sorted(
        Document::from_string(indoc! {"
            jobs:
                build:
                    steps:
                        - uses: docker://alpine:3
                          with:
                              entrypoint: /bin/sh
                              args: --verbose
                              config: /etc/app.conf
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                steps:
                - uses: docker://alpine:3
                  with:
                    entrypoint: /bin/sh
                    args: --verbose
                    config: /etc/app.conf
        "}.to_string()
    )]
    fn test_with_sorter(#[case] source_doc: Document, #[case] expected: String) {
        let result = WithSorter
            .process(source_doc)
            .expect("processing failed")
            .to_string();

        assert_eq!(result, expected);
    }
}
