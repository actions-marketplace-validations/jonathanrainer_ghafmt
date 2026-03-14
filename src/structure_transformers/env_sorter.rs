//! Sorts entries in every `env` mapping alphabetically.
use fyaml::Document;

use crate::structure_transformers::{
    StructureTransformer, for_each_mapping_child, for_each_seq_element,
};

#[derive(Default)]
/// Sorts entries in every `env` mapping alphabetically.
pub(crate) struct EnvSorter;

impl StructureTransformer for EnvSorter {
    fn process(&self, mut doc: Document) -> fyaml::Result<Document> {
        // Sort entries under top level env key
        doc = self.sort_path_to_mapping_alphabetically(doc, "env")?;
        doc = for_each_mapping_child(doc, "jobs", |doc, job_path| {
            // Sort job-level env key
            let doc = self.sort_path_to_mapping_alphabetically(doc, &format!("{job_path}/env"))?;
            // Sort container level env key
            let doc = self
                .sort_path_to_mapping_alphabetically(doc, &format!("{job_path}/container/env"))?;
            // Sort env key for each step if it's defined
            for_each_seq_element(doc, &format!("{job_path}/steps"), |doc, step_path| {
                self.sort_path_to_mapping_alphabetically(doc, &format!("{step_path}/env"))
            })
        })?;
        Ok(doc)
    }

    fn name(&self) -> &'static str {
        "env-sorter"
    }

    fn description(&self) -> &'static str {
        "Sort entries in every 'env' mapping"
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
    #[case::no_env_keys(
        Document::from_string(indoc! {"
            on: push
            jobs:
                build:
                    runs-on: ubuntu-latest
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            on: push
            jobs:
              build:
                runs-on: ubuntu-latest
        "}.to_string()
    )]
    #[case::top_level_env_sorted(
        Document::from_string(indoc! {"
            env:
                ZEBRA: z
                ALPHA: a
                MIDDLE: m
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            env:
              ALPHA: a
              MIDDLE: m
              ZEBRA: z
        "}.to_string()
    )]
    #[case::job_level_env_sorted(
        Document::from_string(indoc! {"
            jobs:
                build:
                    env:
                        NODE_ENV: production
                        CI: true
                        DEBUG: false
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                env:
                  CI: true
                  DEBUG: false
                  NODE_ENV: production
        "}.to_string()
    )]
    #[case::step_level_env_sorted(
        Document::from_string(indoc! {"
            jobs:
                build:
                    steps:
                        - name: Test
                          env:
                              Z_VAR: z
                              A_VAR: a
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                steps:
                - name: Test
                  env:
                    A_VAR: a
                    Z_VAR: z
        "}.to_string()
    )]
    #[case::container_env_sorted(
        Document::from_string(indoc! {"
            jobs:
                build:
                    container:
                        env:
                            POSTGRES_DB: test
                            NODE_ENV: test
                            CI: true
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                container:
                  env:
                    CI: true
                    NODE_ENV: test
                    POSTGRES_DB: test
        "}.to_string()
    )]
    #[case::all_levels_sorted(
        Document::from_string(indoc! {"
            env:
                Z_TOP: z
                A_TOP: a
            jobs:
                build:
                    env:
                        Z_JOB: z
                        A_JOB: a
                    container:
                        env:
                            Z_CTR: z
                            A_CTR: a
                    steps:
                        - name: Step 1
                          env:
                              Z_STEP: z
                              A_STEP: a
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            env:
              A_TOP: a
              Z_TOP: z
            jobs:
              build:
                env:
                  A_JOB: a
                  Z_JOB: z
                container:
                  env:
                    A_CTR: a
                    Z_CTR: z
                steps:
                - name: Step 1
                  env:
                    A_STEP: a
                    Z_STEP: z
        "}.to_string()
    )]
    #[case::multiple_jobs_and_steps(
        Document::from_string(indoc! {"
            jobs:
                build:
                    steps:
                        - name: S1
                          env:
                              Z: z
                              A: a
                        - name: S2
                          env:
                              Y: y
                              B: b
                deploy:
                    env:
                        STAGE: prod
                        ENV: deploy
        "}.to_string()).expect("test input is valid YAML"),
        indoc! {"
            jobs:
              build:
                steps:
                - name: S1
                  env:
                    A: a
                    Z: z
                - name: S2
                  env:
                    B: b
                    Y: y
              deploy:
                env:
                  ENV: deploy
                  STAGE: prod
        "}.to_string()
    )]
    fn test_env_sorter(#[case] source_doc: Document, #[case] expected: String) {
        let result = EnvSorter
            .process(source_doc)
            .expect("processing failed")
            .to_string();

        assert_eq!(result, expected);
    }
}
