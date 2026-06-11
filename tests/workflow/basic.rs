use anyhow::Result;
use serde_json::json;

use konfg::workflow::Workflow;

fn fixture(name: &str) -> String {
    format!("tests/fixtures/{}", name)
}

/// Build a `-i tplfile <path> ... -o <output>` args vector.
fn build_args(inputs: &[&str], output: &str) -> Vec<String> {
    let mut v: Vec<String> = inputs
        .iter()
        .flat_map(|p| ["-i".to_string(), "tplfile".to_string(), p.to_string()])
        .collect();
    v.extend(["-o".to_string(), output.to_string()]);
    v
}

#[test]
fn test_merge_two_yaml_files() -> Result<()> {
    let workflow = Workflow::try_from_args(build_args(
        &[&fixture("base.yaml"), &fixture("override.yaml")],
        "noop",
    ))?;
    let result = workflow.execute()?;

    assert_eq!(result["server"]["host"], json!("0.0.0.0"));
    assert_eq!(result["server"]["port"], json!(9090));
    assert_eq!(result["server"]["debug"], json!(true));
    assert_eq!(result["database"]["url"], json!("postgres://localhost/db"));
    assert_eq!(result["app"]["name"], json!("myapp"));

    Ok(())
}

#[test]
fn test_later_file_overwrites_scalar() -> Result<()> {
    let workflow = Workflow::try_from_args(build_args(
        &[&fixture("base.yaml"), &fixture("override.yaml")],
        "noop",
    ))?;
    let result = workflow.execute()?;

    // base.yaml has port 8080; override.yaml has port 9090
    assert_eq!(result["server"]["port"], json!(9090));

    Ok(())
}

#[test]
fn test_deep_merge_preserves_non_overridden_keys() -> Result<()> {
    let workflow = Workflow::try_from_args(build_args(
        &[&fixture("base.yaml"), &fixture("override.yaml")],
        "noop",
    ))?;
    let result = workflow.execute()?;

    // override.yaml does not touch database; it must survive the merge
    assert_eq!(result["database"]["url"], json!("postgres://localhost/db"));

    Ok(())
}
