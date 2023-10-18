extern crate actix_web;
extern crate serde_json;
extern crate simple_logger;
extern crate toml;
use simple_logger::SimpleLogger;
extern crate log;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate serde_derive;

mod config;
mod envoy_reader;
mod handlers;

use crate::config::Config;
use crate::handlers::{index, metrics};
use actix_web::{server, App};
use anyhow::{bail, Context, Result};
use std::env;

static BUILD_TIME: Option<&'static str> = option_env!("BUILD_TIME");
static GIT_REVISION: Option<&'static str> = option_env!("GIT_REVISION");
static RUST_VERSION: Option<&'static str> = option_env!("RUST_VERSION");
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    SimpleLogger::new().env().init()?;
    let version_info = if BUILD_TIME.is_some() {
        format!(
            "  version   : {}\n  revision  : {}\n  build time: {}\n",
            VERSION,
            GIT_REVISION.unwrap_or(""),
            BUILD_TIME.unwrap()
        )
    } else {
        format!("  version: {}\n", VERSION)
    };

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: envoy-exporter [config_file]");
        println!("\n{}", version_info);
        bail!("Not enough arguments");
    }

    let config = Config::from_file(&args[1]).with_context(|| format!("Reading '{}'", &args[1]))?;

    let addr = format!("0.0.0.0:{}", config.listen_port.unwrap_or(9422));

    println!("Server started: {}", addr);

    server::new(move || {
        App::with_state(config.systems.clone())
            .resource("/", |r| r.f(index))
            .resource("/metrics", |r| r.f(metrics))
    })
    .bind(addr)?
    .run();
    Ok(())
}
