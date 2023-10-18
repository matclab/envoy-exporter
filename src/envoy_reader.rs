use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use anyhow::{Context, Result};
use log;
use crate::config::System;

pub struct EnvoyReader<'a> {
    system: &'a System,
    status: EnvoyStatus,
}

impl<'a> EnvoyReader<'a> {
    pub fn status(system: &'a System) -> Result<EnvoyStatus> {
        let mut reader = EnvoyReader {
            system,
            status: EnvoyStatus::new(),
        };
        reader.production()?;
        reader.consumption()?;
        reader.inverters()?;
        reader.status.online = true;
        Ok(reader.status)
    }

    fn fetch_json(&self, suffix: &str) -> Result<Value> {
        let url = self.system.url.to_owned() + suffix;
        let json: serde_json::Value = ureq::get(&url)
            .set("Authorization", &("Bearer ".to_owned() + &self.system.token))
            .call()?
            .into_json()?;
        Ok(json)
    }

    fn production(&mut self) -> Result<()> {
        let json: Value = self.fetch_json("/api/v1/production")?;
        self.status.watt_hours_lifetime = json["wattHoursLifetime"].as_i64().with_context(
            || { format!("Casting wattHoursLifetime production to i64 {}", json["wattHoursLifetime"]) })?;
        self.status.watt_hours_today = json["wattHoursToday"].as_i64().with_context(
            || { format!("Casting wattHoursToday production to i64 {}", json["wattHoursToday"]) })?;
        self.status.watts_now = json["wattsNow"].as_i64().with_context(
            || { format!("Casting wattsNow production to i64 {}", json["wattsNow"]) })?;
        Ok(())
    }

    fn consumption(&mut self) -> Result<()> {
        let json: Value = self.fetch_json("/api/v1/consumption")?;
        self.status.watt_hours_lifetime_consumption = json["wattHoursLifetime"].as_i64().with_context(
            || { format!("Casting wattHoursLifetime consumption to i64 {}", json["wattHoursLifetime"]) })?;

        self.status.watt_hours_today_consumption = json["wattHoursToday"].as_i64().with_context(
            || { format!("Casting wattHoursToday consumption to i64 {}", json["wattHoursToday"]) })?;
        self.status.watts_now_consumption = json["wattsNow"].as_i64().with_context(
            || { format!("Casting wattsNow consumption to i64 {}", json["wattsNow"]) })?;
        Ok(())
    }

    fn inverters(&mut self) -> anyhow::Result<()> {
        let json: Value = self.fetch_json("/api/v1/production/inverters")?;
        log::debug!("Receive {:?}", json);
        let inverters = json.as_array().with_context(
            || { format!("Casting json to array {:?}", json) })?;
        for inverter in inverters {
            let sn = inverter["serialNumber"].as_str().context(format!("Invalid serial number {:?}", inverter["serialNumber"]))?;
            let watts = inverter["lastReportWatts"].as_i64().with_context(
            || { format!("Casting inverter lastReportWatts to i64 {}", inverter["lastReportWatts"])})?;
            self.status.inverters.insert(sn.to_owned(), watts);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct EnvoyStatus {
    pub online: bool,
    pub watt_hours_lifetime: i64,
    pub watt_hours_today: i64,
    pub watts_now: i64,
    pub watt_hours_lifetime_consumption: i64,
    pub watt_hours_today_consumption: i64,
    pub watts_now_consumption: i64,
    pub inverters: HashMap<String, i64>,
}

impl EnvoyStatus {
    pub fn new() -> EnvoyStatus {
        EnvoyStatus {
            online: false,
            watt_hours_lifetime: 0,
            watt_hours_today: 0,
            watts_now: 0,
            watt_hours_lifetime_consumption: 0,
            watt_hours_today_consumption: 0,
            watts_now_consumption: 0,
            inverters: HashMap::new(),
        }
    }
}
