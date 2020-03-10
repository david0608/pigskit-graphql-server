use clap::{App, Arg, ArgMatches};
use crate::DEFAULT_PORT;

pub fn parse_arguments<'a>() -> ArgMatches<'a> {
    App::new("pikit-server")
        .version("1.0")
        .author("David Wu <david6906817@gmail.com>")
        .about("Pikit server.")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("Set the port that server will listen.")
                .takes_value(true)
                .default_value(DEFAULT_PORT),
        )
        .get_matches()
}