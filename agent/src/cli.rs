use clap::{Arg, ArgMatches, Command};
use std::env;

pub struct Config {
    pub interval: Option<u64>,
}

impl From<ArgMatches> for Config {
    fn from(matches: ArgMatches) -> Self {
        Config {
            interval: matches.get_one::<u64>("watch").copied(),
        }
    }
}

pub(crate) fn configure_cli() -> Config {
    let default_watch_interval = env::var("WATCH_INTERVAL").unwrap_or_else(|_| "60".to_string());
    let matches = Command::new("deploy")
        .version("1.0")
        .author("Marius Kriegerowski")
        .about("deploy and roll back docker containers at ease")
        .arg(
            Arg::new("watch")
                .short('w')
                .long("watch")
                .help(format!(
                    "Watch mode: re-run every N seconds (default: {default_watch_interval})"
                ))
                .value_name("SECONDS")
                .num_args(0..=1)
                .default_missing_value(default_watch_interval)
                .value_parser(clap::value_parser!(u64)),
        )
        .get_matches();
    matches.into()
}
