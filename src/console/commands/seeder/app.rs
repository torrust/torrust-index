//! Program to upload random torrent to a live Index API.
//!
//! Run with:
//!
//! ```text
//! cargo run --bin seeder -- --number-of-torrents <NUMBER_OF_TORRENTS> --user <USER> --password <PASSWORD> --interval <INTERVAL>
//! ```
//!
//! For example:
//!
//! ```text
//! cargo run --bin seeder -- --number-of-torrents 1000 --user admin --password 12345678 --interval 0
//! ```
//!
//! That command would upload 100o random torrents to the Index using the user
//! account admin with password 123456 and waiting 1 second between uploads.
use clap::Parser;
use log::{debug, LevelFilter};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    number_of_torrents: i32,

    #[arg(short, long)]
    user: String,

    #[arg(short, long)]
    password: String,

    #[arg(short, long)]
    interval: i32,
}

/// # Errors
///
/// Will not return any errors for the time being.
pub fn run() -> anyhow::Result<()> {
    setup_logging(LevelFilter::Info);

    let args = Args::parse();

    println!("Number of torrents: {}", args.number_of_torrents);
    println!("User: {}", args.user);
    println!("Password: {}", args.password);
    println!("Interval: {:?}", args.interval);

    /* todo:
        - Use a client to upload a random torrent every "interval" seconds.
    */

    Ok(())
}

fn setup_logging(level: LevelFilter) {
    if let Err(_err) = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}][{}] {}",
                chrono::Local::now().format("%+"),
                record.target(),
                record.level(),
                message
            ));
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()
    {
        panic!("Failed to initialize logging.")
    }

    debug!("logging initialized.");
}
