use clap::{crate_authors, crate_description, crate_version, Arg, ArgAction, Command};
use pretty_env_logger::env_logger::Builder;
use std::env;
use std::io::Write;
use std::process::exit;

use crate::Config;

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
                .short('t')
                .long("test")
                .help("Check the configuration"),
        )
        .arg(
            Arg::new("dry-run")
                .action(ArgAction::SetTrue)
                .long("dry-run")
                .help("Show changes without applying them"),
        )
        .arg(
            Arg::new("backends")
                .action(ArgAction::Append)
                .value_delimiter(',')
                .long("backends")
                .help("Enabled backends"),
        )
        .arg(
            Arg::new("frontends")
                .action(ArgAction::Append)
                .long("frontends")
                .help("Enabled frontends"),
        )
        .version(crate_version!())
        .author(crate_authors!("\n"));

    let args = cli.get_matches();

    setup_logger();

    let config = match Config::with_services(
        args.get_many("backends")
            .expect("at least one backend required")
            .cloned()
            .collect(),
        args.get_many("frontends")
            .expect("at least one frontend required")
            .cloned()
            .collect(),
    )
    .populate_from_env()
    {
        Ok(c) => c,
        Err(err) => {
            println!("{err}");
            exit(2);
        }
    };

    if args.get_flag("check") {
        let (backends, frontends) = config.into_impls();
        tracing::info!(
            backends = backends.len(),
            frontends = frontends.len(),
            "Configuration is valid."
        );
        exit(0);
    }

    config.get_service().sync(args.get_flag("dry-run")).unwrap();
}
