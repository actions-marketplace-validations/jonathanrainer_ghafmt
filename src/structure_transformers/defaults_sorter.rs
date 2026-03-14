//! Sorts top-level and job-specific `defaults` mappings alphabetically.
use fyaml::Document;

use crate::structure_transformers::{StructureTransformer, for_each_mapping_child};

#[derive(Default)]
/// Sorts top-level and job-specific `defaults` mappings alphabetically.
pub(crate) struct DefaultsSorter;

impl StructureTransformer for DefaultsSorter {
    fn process(&self, mut doc: Document) -> fyaml::Result<Document> {
        doc = self.sort_path_to_mapping_alphabetically(doc, "defaults/run")?;
        doc = for_each_mapping_child(doc, "jobs", |doc, job_path| {
            self.sort_path_to_mapping_alphabetically(doc, &format!("{job_path}/defaults/run"))
        })?;
        Ok(doc)
    }

    fn name(&self) -> &'static str {
        "defaults-sorter"
    }

    fn description(&self) -> &'static str {
        "Sort top-level and job specific 'defaults' into alphabetical order"
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
    #[case::no_defaults(
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
    #[case::top_level_defaults_run_sorted(
        Document::from_string(indoc! {"
            defaults:
                run:
                    working-directory: src
                    shell: bash
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            defaults:
              run:
                shell: bash
                working-directory: src
        "}.to_string()
    )]
    #[case::job_level_defaults_run_sorted(
        Document::from_string(indoc! {"
            jobs:
                build:
                    defaults:
                        run:
                            working-directory: src
                            shell: bash
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                defaults:
                  run:
                    shell: bash
                    working-directory: src
        "}.to_string()
    )]
    #[case::both_levels_sorted(
        Document::from_string(indoc! {"
            defaults:
                run:
                    working-directory: root
                    shell: zsh
            jobs:
                build:
                    defaults:
                        run:
                            working-directory: src
                            shell: bash
                deploy:
                    defaults:
                        run:
                            working-directory: deploy
                            shell: sh
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            defaults:
              run:
                shell: zsh
                working-directory: root
            jobs:
              build:
                defaults:
                  run:
                    shell: bash
                    working-directory: src
              deploy:
                defaults:
                  run:
                    shell: sh
                    working-directory: deploy
        "}.to_string()
    )]
    fn test_defaults_sorter(#[case] source_doc: Document, #[case] expected: String) {
        let result = DefaultsSorter
            .process(source_doc)
            .expect("processing failed")
            .to_string();

        assert_eq!(result, expected);
    }
}
