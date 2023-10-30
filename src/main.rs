use simple_logger::SimpleLogger;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate serde_derive;

use crate::config::Config;
use crate::handlers::{index, metrics};
use actix_web::{server, App};
use anyhow::{bail, Context, Result};
use std::env;
use std::sync::Arc;
use std::time::SystemTime;

use rustls::{
    client::{ServerCertVerified, ServerCertVerifier},
    Certificate, ClientConfig, ServerName
};
mod config;
mod envoy_reader;
mod handlers;

static BUILD_TIME: Option<&'static str> = option_env!("BUILD_TIME");
static GIT_REVISION: Option<&'static str> = option_env!("GIT_REVISION");
static RUST_VERSION: Option<&'static str> = option_env!("RUST_VERSION");
static VERSION: &str = env!("CARGO_PKG_VERSION");

struct AcceptAll {}

impl ServerCertVerifier for AcceptAll {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

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
    //    let mut roots = rustls::RootCertStore::empty();
    //    for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
    //        roots.add(&rustls::Certificate(cert.0)).unwrap();
    //    }

    let mut builder = ureq::builder();
        let client_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(AcceptAll {}))
        .with_no_client_auth();
    builder = builder.tls_config(Arc::new(client_config));
    let agent = builder.build();
    let config = Config::from_file(&args[1], agent).with_context(|| format!("Reading '{}'", &args[1]))?;

    let addr = format!("0.0.0.0:{}", config.listen_port.unwrap_or(9422));

    println!("Server started: {}", addr);

    server::new(move || {
        App::with_state(config.clone())
            .resource("/", |r| r.f(index))
            .resource("/metrics", |r| r.f(metrics))
    })
    .bind(addr)?
    .run();
    Ok(())
}
