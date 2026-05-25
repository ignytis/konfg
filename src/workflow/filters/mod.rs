pub mod delete;
pub mod move_filter;

use std::{
    collections::{HashMap, VecDeque},
    sync::LazyLock,
};

use anyhow::Result;
use serde_json::Value;

use crate::{jinja::JinjaEngine, workflow::stage::Stage};

pub const REGISTERED_FILTERS: LazyLock<Vec<Box<dyn BaseFilter>>> = LazyLock::new(|| {
    vec![
        Box::new(delete::DeleteFilter),
        Box::new(move_filter::MoveFilter),
    ]
});

/// Result of attempting to parse tokens by a filter.
pub enum TryParseFilterResult {
    /// Arguments were successfully parsed into Stage
    Success(Stage),
    /// Parser does not support the provided filter
    NotSupported,
    /// Parser supports the provided filter, but an error occurred
    Error(anyhow::Error),
}

/// Base trait for handling filter operations.
pub trait BaseFilter: Send + Sync {
    /// Checks if this filter supports the given kind, e.g. "delete".
    fn supports(&self, kind: &str) -> bool;

    /// Clones the filter into a boxed trait object.
    fn clone_box(&self) -> Box<dyn BaseFilter>;

    /// Attempts to pop args and construct a `Stage`.
    /// Returns `TryParseFilterResult::NotSupported` if the first token is not supported by this filter.
    fn try_parse_args(
        &self,
        args: &mut VecDeque<String>,
        jinja: &JinjaEngine,
    ) -> TryParseFilterResult;
}

/// Trait for handling filter operations.
pub trait Filter: BaseFilter {
    /// Applies the filter to the merged configuration.
    fn apply(&self, args: &HashMap<String, String>, merged: &mut Value) -> Result<()>;
}

impl Clone for Box<dyn BaseFilter> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
