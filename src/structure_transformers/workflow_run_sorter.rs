//! Sorts `workflow_run` keys and their children alphabetically.
use fyaml::Document;

use crate::structure_transformers::{StructureTransformer, for_each_mapping_child};

#[derive(Default)]
/// Sorts `workflow_run` keys and their children alphabetically.
pub(crate) struct WorkflowRunSorter {}

impl StructureTransformer for WorkflowRunSorter {
    fn process(&self, mut doc: Document) -> fyaml::Result<Document> {
        let workflow_run_path = "/on/workflow_run";
        doc = self.sort_path_to_mapping_alphabetically(doc, workflow_run_path)?;
        doc = for_each_mapping_child(doc, workflow_run_path, |doc, child_path| {
            self.sort_seq_at_path_alphabetically(doc, child_path)
        })?;
        Ok(doc)
    }

    fn name(&self) -> &'static str {
        "workflow-run-sorter"
    }

    fn description(&self) -> &'static str {
        "Sort 'workflow_run' keys and their children alphabetically"
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
    #[case::no_workflow_run(
        Document::from_string(indoc! {"
            on: push
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on: push
        "}.to_string()
    )]
    #[case::mapping_keys_sorted(
        Document::from_string(indoc! {"
            on:
                workflow_run:
                    workflows:
                        - Deploy
                    types:
                        - completed
                    branches:
                        - main
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on:
              workflow_run:
                branches:
                - main
                types:
                - completed
                workflows:
                - Deploy
        "}.to_string()
    )]
    #[case::sequence_values_sorted(
        Document::from_string(indoc! {"
            on:
                workflow_run:
                    workflows:
                        - Deploy
                        - Build
                        - Test
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on:
              workflow_run:
                workflows:
                - Build
                - Deploy
                - Test
        "}.to_string()
    )]
    #[case::both_sorted(
        Document::from_string(indoc! {"
            on:
                workflow_run:
                    workflows:
                        - Deploy
                        - Build
                        - Test
                    types:
                        - completed
                        - requested
                    branches:
                        - main
                        - develop
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on:
              workflow_run:
                branches:
                - develop
                - main
                types:
                - completed
                - requested
                workflows:
                - Build
                - Deploy
                - Test
        "}.to_string()
    )]
    #[case::already_sorted(
        Document::from_string(indoc! {"
            on:
                workflow_run:
                    branches:
                        - develop
                    workflows:
                        - Build
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on:
              workflow_run:
                branches:
                - develop
                workflows:
                - Build
        "}.to_string()
    )]
    fn test_workflow_run_sorter(#[case] source_doc: Document, #[case] expected: String) {
        let result = WorkflowRunSorter::default()
            .process(source_doc)
            .expect("processing failed")
            .to_string();

        assert_eq!(result, expected);
    }
}
