mod functions;

use anyhow::Result;
use minijinja::Environment;

pub struct JinjaEngine {
    env: Environment<'static>,
}

impl JinjaEngine {
    pub fn new() -> Self {
        let mut env = Environment::new();
        register_functions(&mut env);
        Self { env }
    }

    pub fn render<S: Into<String>>(&self, template: S, ctx: &serde_json::Value) -> Result<String> {
        match self
            .env
            .render_named_str("configuration", template.into().as_str(), ctx)
        {
            Ok(s) => Ok(s),
            Err(e) => Err(e.into()),
        }
    }
}

fn register_functions(env: &mut Environment) {
    env.add_function("env", functions::env);
    env.add_function("md5", functions::md5);
    env.add_function("sha256", functions::sha256);
    env.add_function("sha512", functions::sha512);
}
