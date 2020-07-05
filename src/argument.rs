use clap::{App, Arg, ArgMatches};

pub fn parse_arguments<'a>() -> ArgMatches<'a> {
    App::new("pigskit-server")
        .version("1.0")
        .author("David Wu <david6906817@gmail.com>")
        .about("Pigskit server.")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("Set the port that server will listen.")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("dev")
                .help("run server in development mode.")
                .short("d"),
        )
        .get_matches()
}

pub fn args_port(args: &ArgMatches) -> Option<u16> {
    if let Some(port) = args.value_of("port") {
        if let Ok(port) = port.parse::<u16>() {
            Some(port)
        } else {
            None
        }
    } else {
        None
    }
}