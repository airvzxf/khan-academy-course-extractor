use clap::Parser;

/// Command-line arguments for the application.
#[derive(Parser)]
pub struct Args {
    /// Directory path
    #[clap(short, long, default_value = ".")]
    pub path: String,

    /// File prefix
    #[clap(short = 'e', long, default_value = "")]
    pub prefix: String,
}
