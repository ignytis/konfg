pub mod delete;

use std::{
    collections::VecDeque,
    sync::LazyLock,
};

use anyhow::Result;

use crate::{
    workflow::stage::StageExecutionContext,
};

pub type FilterCreatorFn = fn(VecDeque<String>) -> Result<Box<dyn Filter>>;

pub const REGISTERED_FILTERS: LazyLock<Vec<(&'static str, FilterCreatorFn)>> = LazyLock::new(|| {
    vec![
        (delete::KIND, delete::DeleteFilter::new_from_args),
    ]
});

/// Trait for handling filter operations.
pub trait Filter {
    /// Applies the filter to the merged configuration.
    fn apply(
        &self,
        context: &mut StageExecutionContext,
    ) -> Result<()>;
}
