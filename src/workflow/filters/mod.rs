pub mod delete;
pub mod merge_strategies_reset;
pub mod move_filter;
pub mod stash;

use std::sync::LazyLock;

use anyhow::Result;

use crate::workflow::stage::{StageArgs, StageExecutionContext};

pub type FilterCreatorFn = fn(StageArgs) -> Result<Box<dyn Filter>>;

pub const REGISTERED_FILTERS: LazyLock<Vec<(&'static str, FilterCreatorFn)>> =
    LazyLock::new(|| {
        vec![
            (delete::KIND, delete::DeleteFilter::new_from_args),
            (
                merge_strategies_reset::KIND,
                merge_strategies_reset::MergeStrategiesResetFilter::new_from_args,
            ),
            (move_filter::KIND, move_filter::MoveFilter::new_from_args),
            (stash::KIND, stash::StashFilter::new_from_args),
        ]
    });

/// Trait for handling filter operations.
pub trait Filter {
    /// Applies the filter to the merged configuration.
    fn apply(&self, context: &mut StageExecutionContext) -> Result<()>;
}
