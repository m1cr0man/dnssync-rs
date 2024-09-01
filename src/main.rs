pub mod cloudflare;
pub mod common;
pub mod config;
pub mod headscale;
pub mod machinectl;
pub mod service;

pub use config::*;

#[cfg(feature = "cli")]
mod cli;

fn main() {
    #[cfg(not(feature = "cli"))]
    panic!("cli feature is not enabled");
    #[cfg(feature = "cli")]
    cli::main()
}
