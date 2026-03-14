//! Sorts `needs` entries alphabetically within each job.
use fyaml::Document;

use crate::structure_transformers::{StructureTransformer, for_each_mapping_child};

#[derive(Default)]
/// Sorts `needs` entries alphabetically within each job.
pub(crate) struct NeedsSorter {}
impl StructureTransformer for NeedsSorter {
    fn process(&self, mut doc: Document) -> fyaml::Result<Document> {
        doc = for_each_mapping_child(doc, "jobs", |doc, job_path| {
            self.sort_seq_at_path_alphabetically(doc, &format!("{job_path}/needs"))
        })?;
        Ok(doc)
    }

    fn name(&self) -> &'static str {
        "needs-sorter"
    }

    fn description(&self) -> &'static str {
        "Sort 'needs' entries into alphabetical order"
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
    #[case::no_needs(
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
    #[case::scalar_needs_unchanged(
        Document::from_string(indoc! {"
            jobs:
                deploy:
                    needs: build
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              deploy:
                needs: build
        "}.to_string()
    )]
    #[case::flow_sequence_sorted(
        Document::from_string(indoc! {"
            jobs:
                deploy:
                    needs: [test, lint, build]
        "}.to_string()).expect("test input is valid YAML"),
        "jobs:\n  deploy:\n    needs: [\n      build,\n      lint,\n      test\n      ]\n".to_string()
    )]
    #[case::already_sorted(
        Document::from_string(indoc! {"
            jobs:
                deploy:
                    needs: [a, b, c]
        "}.to_string()).expect("test input is valid YAML"),
        "jobs:\n  deploy:\n    needs: [\n      a,\n      b,\n      c\n      ]\n".to_string()
    )]
    #[case::multiple_jobs(
        Document::from_string(indoc! {"
            jobs:
                build:
                    runs-on: ubuntu-latest
                test:
                    needs: [build]
                deploy:
                    needs: [test, lint, build]
        "}.to_string()).expect("test input is valid YAML"),
        "jobs:\n  build:\n    runs-on: ubuntu-latest\n  test:\n    needs: [\n      build\n      ]\n  deploy:\n    needs: [\n      build,\n      lint,\n      test\n      ]\n".to_string()
    )]
    #[case::block_sequence_sorted(
        Document::from_string(indoc! {"
            jobs:
                deploy:
                    needs:
                        - test
                        - build
                        - lint
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              deploy:
                needs:
                - build
                - lint
                - test
        "}.to_string()
    )]
    fn test_needs_sorter(#[case] source_doc: Document, #[case] expected: String) {
        let result = NeedsSorter::default()
            .process(source_doc)
            .expect("processing failed")
            .to_string();

        assert_eq!(result, expected);
    }
}
