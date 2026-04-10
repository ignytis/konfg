use anyhow::Result;
use minijinja::Environment;
use serde_json::Value;

pub struct JinjaEngine {
    env: Environment<'static>,
}

impl JinjaEngine {
    pub fn new() -> Self {
        let env = Environment::new();
        Self { env }
    }

    pub fn render<S: Into<String>>(
        &self,
        template: S,
        ctx: &serde_json::Map<String, Value>,
    ) -> Result<String> {
        match self
            .env
            .render_named_str("configuration", template.into().as_str(), ctx)
        {
            Ok(s) => Ok(s),
            Err(e) => Err(e.into()),
        }
    }
}
