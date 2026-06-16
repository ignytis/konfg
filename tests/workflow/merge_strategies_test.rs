use anyhow::Result;
use serde_json::json;

use konfg::workflow::Workflow;

fn example(name: &str) -> String {
    format!("examples/300_merge_strategies/{}", name)
}

fn s(v: &str) -> String {
    v.to_string()
}

/// 1. Default (simple) merge strategy: maps merged recursively, arrays appended.
#[test]
fn test_simple_merge() -> Result<()> {
    let args = vec![
        s("-i"),
        s("file"),
        example("config_a.yaml"),
        s("-i"),
        s("file"),
        example("config_b.yaml"),
        s("-o"),
        s("noop"),
    ];
    let result = Workflow::try_from_args(args)?.execute()?;

    assert_eq!(result["app"]["name"], json!("My App"));
    assert_eq!(result["app"]["database"]["host"], json!("localhost"));
    assert_eq!(result["app"]["database"]["port"], json!(5433));
    // arrays are appended: 2 from a + 2 from b = 4 elements
    assert_eq!(result["app"]["features"].as_array().unwrap().len(), 4);

    Ok(())
}

/// 2. merge_by_key strategy: -m must precede the second input to take effect.
/// Array elements matched by "name" are merged rather than appended.
#[test]
fn test_merge_by_key() -> Result<()> {
    let args = vec![
        s("-i"),
        s("file"),
        example("config_a.yaml"),
        s("-m"),
        s("app.features"),
        s("merge_by_key"),
        s("name"),
        s("-i"),
        s("file"),
        example("config_b.yaml"),
        s("-o"),
        s("noop"),
    ];
    let result = Workflow::try_from_args(args)?.execute()?;

    let features = result["app"]["features"].as_array().unwrap();
    // feature1 (only in a), feature2 (merged), feature3 (only in b) = 3 elements
    assert_eq!(features.len(), 3);

    let feature2 = features.iter().find(|f| f["name"] == "feature2").unwrap();
    assert_eq!(feature2["enabled"], json!(true));
    assert_eq!(feature2["details"], json!("more info"));

    Ok(())
}

/// 3. overwrite strategy then reset to simple: database from b fully replaces a, then simple merge applies.
#[test]
fn test_overwrite_then_simple() -> Result<()> {
    let args = vec![
        s("-i"),
        s("file"),
        example("config_a.yaml"),
        s("-m"),
        s("app.database"),
        s("overwrite"),
        s("-i"),
        s("file"),
        example("config_b.yaml"),
        s("-m"),
        s("app.database"),
        s("simple"),
        s("-o"),
        s("noop"),
    ];
    let result = Workflow::try_from_args(args)?.execute()?;

    // overwrite drops host from a, then simple merge adds port from b
    assert!(result["app"]["database"]["host"].is_null());
    assert_eq!(result["app"]["database"]["port"], json!(5433));

    Ok(())
}

/// 4. Combining merge_by_key and overwrite strategies (both -m before second input).
#[test]
fn test_merge_by_key_and_overwrite() -> Result<()> {
    let args = vec![
        s("-i"),
        s("file"),
        example("config_a.yaml"),
        s("-m"),
        s("app.features"),
        s("merge_by_key"),
        s("name"),
        s("-m"),
        s("app.database"),
        s("overwrite"),
        s("-i"),
        s("file"),
        example("config_b.yaml"),
        s("-o"),
        s("noop"),
    ];
    let result = Workflow::try_from_args(args)?.execute()?;

    // features merged by key: 3 elements
    assert_eq!(result["app"]["features"].as_array().unwrap().len(), 3);
    // database overwritten: only b's fields remain
    assert!(result["app"]["database"]["host"].is_null());
    assert_eq!(result["app"]["database"]["port"], json!(5433));

    Ok(())
}
