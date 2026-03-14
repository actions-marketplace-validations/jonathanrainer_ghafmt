//! Constants defining canonical key orderings and event lists for GHA workflow formatting.

/// Canonical order for top-level GitHub Actions workflow keys.
pub(crate) const TOP_LEVEL_KEY_ORDERING: [&str; 8] = [
    "name",
    "run-name",
    "on",
    "permissions",
    "concurrency",
    "env",
    "defaults",
    "jobs",
];

/// Canonical order for keys within a `workflow_dispatch` or `workflow_call` input block.
pub(crate) const INPUT_ORDER: [&str; 5] = ["description", "type", "required", "default", "options"];
/// Canonical order for keys within a `workflow_call` output block.
pub(crate) const OUTPUT_ORDER: [&str; 2] = ["description", "value"];
/// Canonical order for keys within a `workflow_call` secret block.
pub(crate) const SECRET_ORDER: [&str; 2] = ["description", "required"];
/// Top-level section keys present under `workflow_call` or `workflow_dispatch`.
pub(crate) const WORKFLOW_KEYS: [&str; 3] = ["inputs", "outputs", "secrets"];

/// Exhaustive list of GitHub Actions event types used to build filter paths.
pub(crate) const EVENT_TYPES: [&str; 34] = [
    "branch_protection_rule",
    "check_run",
    "check_suite",
    "create",
    "delete",
    "deployment",
    "deployment_status",
    "discussion",
    "discussion_comment",
    "fork",
    "gollum",
    "image_version",
    "issue_comment",
    "issues",
    "label",
    "merge_group",
    "milestone",
    "page_build",
    "public",
    "pull_request",
    "pull_request_comment",
    "pull_request_review",
    "pull_request_review_comment",
    "pull_request_target",
    "push",
    "registry_package",
    "release",
    "repository_dispatch",
    "schedule",
    "status",
    "watch",
    "workflow_call",
    "workflow_dispatch",
    "workflow_run",
];
