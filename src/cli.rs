use clap::{Arg, ArgMatches, Command};

pub struct Config {
    pub interval: Option<u64>,
}

impl Config {
    pub(crate) fn watch(&self) -> bool {
        self.interval.is_some()
    }
}

impl From<ArgMatches> for Config {
    fn from(matches: ArgMatches) -> Self {
        Config {
            interval: matches.get_one::<u64>("watch").copied(),
        }
    }
}

pub(crate) fn configure_cli() -> Config {
    let matches = Command::new("deploy")
        .version("1.0")
        .author("Marius Kriegerowski")
        .about("deploy and roll back docker containers at ease")
        .arg(
            Arg::new("watch")
                .short('w')
                .long("watch")
                .help("Watch mode: re-run every N seconds (default: 60)")
                .value_name("SECONDS")
                .num_args(0..=1)
                .default_missing_value("60")
                .value_parser(clap::value_parser!(u64)),
        )
        .get_matches();
    matches.into()
}
