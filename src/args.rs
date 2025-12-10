use clap::{command, Parser};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(long)]
    pub ip: String,

    #[clap(long)]
    pub port: String,

    #[clap(long)]
    pub path: String
}