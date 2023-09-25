use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// network port to use
    ///
    /// This option allows the user to specify the network port to be used by the application.
    #[arg(short, long, default_value_t = 8080, value_name = "PORT", value_parser=clap::value_parser!(u16).range(1024..65535))]
    pub port: u16,

    /// Path to look for configuration files.
    ///
    /// This option allows the user to specify a custom search path for configuration files.
    /// By default, it will look for files in the current directory.
    #[arg(short, long, default_value_t = String::from("./"), value_name = "SEARCH_PATH")]
    pub search_path: String,

    /// Size of the cache.
    ///
    /// This option allows you to cache the HttpResponse for routes.
    #[arg(short, long, default_value_t = 20, value_name = "CACHE_SIZE")]
    pub cache: usize,
}
