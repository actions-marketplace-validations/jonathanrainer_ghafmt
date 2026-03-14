//! Sorts all `on` trigger entries, appropriate to their underlying YAML type.
use fyaml::{Document, NodeType};

use crate::structure_transformers::{StructureTransformer, for_each_mapping_child};

#[derive(Default)]
/// Sorts all `on` trigger entries, appropriate to their underlying YAML type.
pub(crate) struct OnSorter {}
impl StructureTransformer for OnSorter {
    fn process(&self, mut doc: Document) -> fyaml::Result<Document> {
        match doc.at_path("/on") {
            None => return Ok(doc),
            Some(node) => match node.kind() {
                // If 'on' is a scalar then just return the doc, there's nothing to do
                NodeType::Scalar => return Ok(doc),
                // If it's a sequence of event names, sort it alphabetically
                NodeType::Sequence => doc = self.sort_seq_at_path_alphabetically(doc, "on")?,
                // If it's a mapping, sort its top-level keys and then, for each child, sort its
                // keys alphabetically
                NodeType::Mapping => {
                    doc = self.sort_path_to_mapping_alphabetically(doc, "on")?;

                    doc = for_each_mapping_child(doc, "on", |doc, key| {
                        self.sort_path_to_mapping_alphabetically(doc, key)
                    })?;
                }
            },
        }

        Ok(doc)
    }

    fn name(&self) -> &'static str {
        "on-sorter"
    }

    fn description(&self) -> &'static str {
        "Sort all 'on' entries, appropriate to their underlying type"
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
    #[case::no_on_key(
        Document::from_string(indoc! {"
            jobs:
                build:
                    runs-on: ubuntu-latest
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                runs-on: ubuntu-latest
        "}.to_string()
    )]
    #[case::scalar_on(
        Document::from_string(indoc! {"
            on: push
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on: push
        "}.to_string()
    )]
    #[case::sequence_sorted(
        Document::from_string(indoc! {"
            on:
                - workflow_dispatch
                - push
                - pull_request
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on:
            - pull_request
            - push
            - workflow_dispatch
        "}.to_string()
    )]
    #[case::sequence_already_sorted(
        Document::from_string(indoc! {"
            on:
                - pull_request
                - push
                - workflow_dispatch
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on:
            - pull_request
            - push
            - workflow_dispatch
        "}.to_string()
    )]
    #[case::mapping_triggers_sorted(
        Document::from_string(indoc! {"
            on:
                workflow_dispatch:
                    inputs:
                        debug:
                            description: Enable debug logging
                            type: boolean
                push:
                    branches:
                        - main
                pull_request:
                    branches:
                        - main
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on:
              pull_request:
                branches:
                - main
              push:
                branches:
                - main
              workflow_dispatch:
                inputs:
                  debug:
                    description: Enable debug logging
                    type: boolean
        "}.to_string()
    )]
    #[case::mapping_trigger_subkeys_sorted(
        Document::from_string(indoc! {"
            on:
                push:
                    tags:
                        - 'v*'
                    paths:
                        - 'src/**'
                    branches:
                        - main
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on:
              push:
                branches:
                - main
                paths:
                - 'src/**'
                tags:
                - 'v*'
        "}.to_string()
    )]
    #[case::mapping_both_levels_sorted(
        Document::from_string(indoc! {"
            on:
                push:
                    tags:
                        - 'v*'
                    branches:
                        - main
                pull_request:
                    paths:
                        - 'src/**'
                    branches:
                        - main
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on:
              pull_request:
                branches:
                - main
                paths:
                - 'src/**'
              push:
                branches:
                - main
                tags:
                - 'v*'
        "}.to_string()
    )]
    #[case::mapping_already_sorted(
        Document::from_string(indoc! {"
            on:
                pull_request:
                    branches:
                        - main
                push:
                    branches:
                        - main
                    tags:
                        - 'v*'
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on:
              pull_request:
                branches:
                - main
              push:
                branches:
                - main
                tags:
                - 'v*'
        "}.to_string()
    )]
    fn test_on_sorter(#[case] source_doc: Document, #[case] expected: String) {
        let result = OnSorter::default()
            .process(source_doc)
            .expect("processing failed")
            .to_string();

        assert_eq!(result, expected);
    }
}
