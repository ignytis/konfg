use clap::Parser;

/// CLI arguments
#[derive(Parser)]
#[command(name = "konfg", about = "Configuration builder")]
pub struct Cli {
    #[arg(required = true)]
    pub sources: Vec<String>,

    #[arg(short = 'p', long = "param")]
    pub params: Vec<String>,

    #[arg(short = 'o', long = "output", default_value = "stdout-yaml://")]
    pub output: String,
}