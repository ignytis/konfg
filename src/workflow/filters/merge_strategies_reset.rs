use std::collections::VecDeque;

use anyhow::Result;

use crate::{workflow::filters::Filter, workflow::stage::StageExecutionContext};

/// The identifier for the merge strategies reset filter.
pub const KIND: &str = "merge_strategies_reset";

/// Filter that resets the merge strategy to default.
#[derive(Clone, Default)]
pub struct MergeStrategiesResetFilter {}

impl MergeStrategiesResetFilter {
    /// Creates a new instance of the filter from the given arguments.
    pub fn new_from_args(_args: VecDeque<String>) -> Result<Box<dyn Filter>> {
        Ok(Box::new(MergeStrategiesResetFilter {}))
    }
}

impl Filter for MergeStrategiesResetFilter {
    /// Applies the filter by clearing all merge strategies in the context.
    fn apply(&self, context: &mut StageExecutionContext) -> Result<()> {
        context.merge_strategies.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        workflow::filters::Filter,
        workflow::filters::merge_strategies_reset::MergeStrategiesResetFilter,
        workflow::stage::StageExecutionContext,
    };

    use serde_json::json;

    #[test]
    fn test_merge_strategies_reset_filter_apply() {
        let filter = MergeStrategiesResetFilter {};
        let mut merge_strategies = HashMap::new();
        merge_strategies.insert("a".to_string(), vec!["overwrite".to_string()].into());

        let mut context = StageExecutionContext {
            current_config: json!({}),
            stash: HashMap::new(),
            merge_strategies,
        };

        assert!(!context.merge_strategies.is_empty());
        filter.apply(&mut context).unwrap();
        assert!(context.merge_strategies.is_empty());
    }
}
