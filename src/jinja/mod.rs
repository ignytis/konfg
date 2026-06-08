mod functions;

use anyhow::Result;
use minijinja::Environment;

#[derive(Clone)]
pub struct JinjaEngine {
    env: Environment<'static>,
}

impl JinjaEngine {
    pub fn new() -> Self {
        let mut env = Environment::new();
        register_functions(&mut env);
        env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
        Self { env }
    }

    pub fn render(
        &self,
        template: &str,
        path: &str,
        ctx: &serde_json::Value,
    ) -> Result<String> {
        match self
            .env
            .render_named_str(path, template, ctx)
        {
            Ok(s) => Ok(s),
            Err(e) => anyhow::bail!(
                "An error occurred while rendering the template:\n{}",
                e.display_debug_info().to_string()
            ),
        }
    }
}

fn register_functions(env: &mut Environment) {
    env.add_function("command", functions::command);
    env.add_function("env", functions::env);
    env.add_function("md5", functions::md5);
    env.add_function("sha256", functions::sha256);
    env.add_function("sha512", functions::sha512);
}
