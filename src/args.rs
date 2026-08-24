use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Create the default configuration
    #[arg(long)]
    pub create_config: bool,
}
