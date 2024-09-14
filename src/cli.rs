use clap::{crate_authors, crate_description, crate_version, Arg, ArgAction, Command};
use pretty_env_logger::env_logger::Builder;
use std::env;
use std::io::Write;
use std::{error::Error, process::exit};

use crate::Config;

fn parse_config<'a, T: serde::Deserialize<'a>>(prefix: &str) -> Result<T, Box<dyn Error>> {
    let cfg_source = config::Config::builder()
        .add_source(
            config::Environment::with_prefix(prefix)
                .convert_case(config::Case::ScreamingSnake)
                .try_parsing(true),
        )
        .build()?;

    cfg_source.try_deserialize().map_err(|err| {
        tracing::error!("Error in the provided configuration: {}", err);
        exit(2);
    })
}

fn set_logger_level(b: &mut Builder) {
    let mut b = b;
    if env::var("RUST_LOG").is_err() {
        b = b.filter_level(log::LevelFilter::Info)
    }
    b.init();
}

fn setup_logger() {
    // Adapted from env_logger examples. <3 Systemd support
    match std::env::var("RUST_LOG_STYLE") {
        Ok(s) if s == "SYSTEMD" => {
            let builder = &mut pretty_env_logger::env_logger::builder();
            builder.format(|buf, record| {
                writeln!(
                    buf,
                    "<{}>{}: {}",
                    match record.level() {
                        log::Level::Error => 3,
                        log::Level::Warn => 4,
                        log::Level::Info => 6,
                        log::Level::Debug => 7,
                        log::Level::Trace => 7,
                    },
                    record.target(),
                    record.args()
                )
            });
            set_logger_level(builder);
        }
        _ => {
            let builder = &mut pretty_env_logger::formatted_builder();
            set_logger_level(builder);
        }
    };
}

pub(crate) fn main() {
    let cli = Command::new("DNSSync")
        .about(format!(
            "{}\n{} {}",
            crate_description!(),
            "Configuration is managed using environment variables.",
            "See the docs for more information.",
        ))
        .arg(
            Arg::new("check")
                .action(ArgAction::SetTrue)
                .short('c')
                .long("check")
                .help("Check the configuration"),
        )
        .version(crate_version!())
        .author(crate_authors!("\n"));

    let args = cli.get_matches();

    setup_logger();

    let config: Config = parse_config("DNSSYNC").unwrap();

    if args.get_flag("check") {
        tracing::info!("Configuration is valid.");
        exit(0);
    }

    config.get_service().sync().unwrap();
}
