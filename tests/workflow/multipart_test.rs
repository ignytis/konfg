use anyhow::Result;
use serde_json::json;

use konfg::workflow::Workflow;

fn example(name: &str) -> String {
    format!("examples/200_multipart_config/{}", name)
}

fn s(v: &str) -> String {
    v.to_string()
}

/// Replicates the command from examples/200_multipart_config/run.sh.
/// MY_EXAMPLE__FEATURE_FLAGS__MY_BLEEDING_EDGE_FEATURE is set to "1" via std::env.
#[test]
fn test_multipart_config() -> Result<()> {
    // SAFETY: tests run single-threaded; no concurrent env access
    unsafe {
        std::env::set_var("MY_EXAMPLE__FEATURE_FLAGS__MY_BLEEDING_EDGE_FEATURE", "1");
    }

    #[rustfmt::skip]
    let args = vec![
        s("-i"), s("file"),    example("values.yaml"),
        s("-i"), s("env"),     s("MY_EXAMPLE"),
        s("-f"), s("stash"),   s("push"), s("--preserve"), s("values"),
        s("-f"), s("move"),    s("."), s("values"),
        s("-i"), s("tplfile"), example("mixin.yaml"),
        s("-f"), s("delete"),  s("values"),
        s("-f"), s("stash"),   s("push"), s("mixin"),
        s("-f"), s("stash"),   s("pop"), s("values"), s("_values"),
        s("-i"), s("tplfile"), example("config_0_global.yaml"),
        s("-i"), s("tplfile"), example("config_1_env.yaml"),
        s("-f"), s("stash"),   s("pop"), s("mixin"), s("_imported_mixin"),
        s("-i"), s("tplfile"), example("config_2_regional.yaml"),
        s("-f"), s("delete"),  s("_values"),
        s("-f"), s("delete"),  s("_imported_mixin"),
        s("-o"), s("noop"),
    ];

    let result = Workflow::try_from_args(args)?.execute()?;

    // From config_0_global.yaml (via _values.base_url = "example.com")
    assert_eq!(result["project"]["name"], json!("Example project"));
    assert_eq!(result["project"]["domains"]["base"], json!("example.com"));
    assert_eq!(result["feature_flags"]["core"], json!(true));

    // From config_1_env.yaml
    assert_eq!(result["runtime"]["mode"], json!("development"));

    // From config_2_regional.yaml
    assert_eq!(result["runtime"]["location"], json!("London"));
    assert_eq!(
        result["project"]["domains"]["cdn"],
        json!("static.eu.example.com")
    );
    assert_eq!(
        result["project"]["domains"]["awesome_website"],
        json!("https://www.example.com")
    );
    assert_eq!(
        result["feature_flags"]["my_bleeding_edge_feature"],
        json!(1)
    );
    assert_eq!(
        result["my_bleeding_edge_feature"]["mode"],
        json!("some_value_here")
    );

    // Temporary helper attributes must be cleaned up
    assert!(result["_values"].is_null());
    assert!(result["_imported_mixin"].is_null());

    Ok(())
}
