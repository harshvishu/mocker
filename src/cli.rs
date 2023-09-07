use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Optional name to operate on
    // pub name: Option<String>,

    /// network port to use
    #[arg(short, long, default_value_t = 8080, value_name = "PORT", value_parser=clap::value_parser!(u16).range(1024..65535))]
    pub port: u16,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}
