use actix_web::{http::ContentEncoding, HttpRequest, HttpResponse};
use crate::config::System;
use crate::envoy_reader::EnvoyReader;
use prometheus;
use prometheus::{Encoder, GaugeVec, IntGaugeVec, TextEncoder};

use crate::GIT_REVISION;
use crate::RUST_VERSION;
use crate::VERSION;

lazy_static! {
    static ref BUILD_INFO: GaugeVec = register_gauge_vec!(
        "envoy_build_info",
        "A metric with a constant '1' value labeled by version, revision and rustversion",
        &["version", "revision", "rustversion"]
    ).unwrap();
    static ref ONLINE: IntGaugeVec = register_int_gauge_vec!(
        "envoy_online",
        "Metric scraping successful",
        &["host", "envoy"]
    ).unwrap();
    static ref CURRENT_WATTS: IntGaugeVec = register_int_gauge_vec!(
        "envoy_current_watts",
        "Number of watts being produced",
        &["host", "envoy"]
    ).unwrap();
    static ref TODAY_WATT_HOURS: IntGaugeVec = register_int_gauge_vec!(
        "envoy_today_watt_hours",
        "Number of watt hours produced today",
        &["host", "envoy"]
    ).unwrap();
    static ref LIFETIME_WATT_HOURS: IntGaugeVec = register_int_gauge_vec!(
        "envoy_lifetime_watt_hours",
        "Number of watt hours ever produced",
        &["host", "envoy"]
    ).unwrap();
    static ref CURRENT_WATTS_CONSUMPTION: IntGaugeVec = register_int_gauge_vec!(
        "envoy_current_watts_consumption",
        "Number of watts being consumed",
        &["host", "envoy"]
    ).unwrap();
    static ref TODAY_WATT_HOURS_CONSUMPTION: IntGaugeVec = register_int_gauge_vec!(
        "envoy_today_watt_hours_consumed",
        "Number of watt hours consumed today",
        &["host", "envoy"]
    ).unwrap();
    static ref LIFETIME_WATT_HOURS_CONSUMPTION: IntGaugeVec = register_int_gauge_vec!(
        "envoy_lifetime_watt_hours_consumed",
        "Number of watt hours ever consumed",
        &["host", "envoy"]
    ).unwrap();
    static ref INVERTER_LAST_WATTS: IntGaugeVec = register_int_gauge_vec!(
        "envoy_inverter_last_watts",
        "Number of watts last reported produced by an inverter",
        &["host", "envoy", "inverter"]
    ).unwrap();
}

static LANDING_PAGE: &'static str = "<html>
<head><title>Enphase Envoy Exporter</title></head>
<body>
<h1>Enphase Envoy Exporter</h1>
<p><a href=\"/metrics\">Metrics</a></p>
</body>
";

pub fn index(_req: &HttpRequest<Vec<System>>) -> HttpResponse {
    HttpResponse::Ok()
        .content_encoding(ContentEncoding::Auto)
        .content_type("text/html")
        .body(LANDING_PAGE)
}

pub fn metrics(req: &HttpRequest<Vec<System>>) -> HttpResponse {
    for sys in req.state().clone() {
        let status = match EnvoyReader::status(&sys) {
            Ok(x) => x,
            Err(x) => {
                ONLINE.with_label_values(&[&sys.host, &sys.sn]).set(0);
                println!("{} failed: {}", sys.host, x);
                continue;
            }
        };
        ONLINE
            .with_label_values(&[&sys.host, &sys.sn])
            .set(if status.online { 1 } else { 0 });
        CURRENT_WATTS
            .with_label_values(&[&sys.host, &sys.sn])
            .set(status.watts_now);
        LIFETIME_WATT_HOURS
            .with_label_values(&[&sys.host, &sys.sn])
            .set(status.watt_hours_lifetime);
        TODAY_WATT_HOURS
            .with_label_values(&[&sys.host, &sys.sn])
            .set(status.watt_hours_today);
        CURRENT_WATTS_CONSUMPTION
            .with_label_values(&[&sys.host, &sys.sn])
            .set(status.watts_now_consumption);
        LIFETIME_WATT_HOURS_CONSUMPTION
            .with_label_values(&[&sys.host, &sys.sn])
            .set(status.watt_hours_lifetime_consumption);
        TODAY_WATT_HOURS_CONSUMPTION
            .with_label_values(&[&sys.host, &sys.sn])
            .set(status.watt_hours_today_consumption);
        for (inverter_serial, watts) in status.inverters {
            INVERTER_LAST_WATTS
                .with_label_values(&[&sys.host, &sys.sn, inverter_serial.as_str()])
                .set(watts);
        }
    }
    let git_revision = GIT_REVISION.unwrap_or("");
    let rust_version = RUST_VERSION.unwrap_or("");
    BUILD_INFO
        .with_label_values(&[&VERSION, &git_revision, &rust_version])
        .set(1.0);

    let metric_families = prometheus::gather();
    let encoder = TextEncoder::new();
    let mut buffer : Vec<u8> = vec![];
    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => () ,
        Err(_) => buffer="['Unable to encode']".into(),
    };

    HttpResponse::Ok()
        .content_encoding(ContentEncoding::Auto)
        .content_type(encoder.format_type())
        .body(buffer)
}
