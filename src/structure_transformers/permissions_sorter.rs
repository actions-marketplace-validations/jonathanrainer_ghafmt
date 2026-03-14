//! Sorts `permissions` mappings at top-level and job-level.
use fyaml::Document;

use crate::structure_transformers::{StructureTransformer, for_each_mapping_child};

#[derive(Default)]
/// Sorts `permissions` at top-level and job-level alphabetically.
pub(crate) struct PermissionsSorter;

impl StructureTransformer for PermissionsSorter {
    fn process(&self, mut doc: Document) -> fyaml::Result<Document> {
        doc = self.sort_path_to_mapping_alphabetically(doc, "permissions")?;
        doc = for_each_mapping_child(doc, "jobs", |doc, job_path| {
            self.sort_path_to_mapping_alphabetically(doc, &format!("{job_path}/permissions"))
        })?;
        Ok(doc)
    }

    fn name(&self) -> &'static str {
        "permissions-sorter"
    }

    fn description(&self) -> &'static str {
        "Sort 'permissions' at top-level and job-level"
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
    #[case::no_permissions(
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
    #[case::top_level_permissions_sorted(
        Document::from_string(indoc! {"
            permissions:
                statuses: write
                contents: read
                id-token: write
                actions: read
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            permissions:
              actions: read
              contents: read
              id-token: write
              statuses: write
        "}.to_string()
    )]
    #[case::job_level_permissions_sorted(
        Document::from_string(indoc! {"
            jobs:
                deploy:
                    permissions:
                        id-token: write
                        contents: read
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              deploy:
                permissions:
                  contents: read
                  id-token: write
        "}.to_string()
    )]
    #[case::both_levels_sorted(
        Document::from_string(indoc! {"
            permissions:
                pull-requests: write
                contents: read
            jobs:
                deploy:
                    permissions:
                        id-token: write
                        contents: read
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            permissions:
              contents: read
              pull-requests: write
            jobs:
              deploy:
                permissions:
                  contents: read
                  id-token: write
        "}.to_string()
    )]
    #[case::already_sorted(
        Document::from_string(indoc! {"
            permissions:
                actions: read
                contents: read
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            permissions:
              actions: read
              contents: read
        "}.to_string()
    )]
    #[case::scalar_permissions_unchanged(
        Document::from_string(indoc! {"
            permissions: read-all
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            permissions: read-all
        "}.to_string()
    )]
    fn test_permissions_sorter(#[case] source_doc: Document, #[case] expected: String) {
        let result = PermissionsSorter
            .process(source_doc)
            .expect("processing failed")
            .to_string();

        assert_eq!(result, expected);
    }
}
