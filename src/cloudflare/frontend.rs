use serde::de::DeserializeOwned;
use snafu::prelude::*;

use crate::common::{Frontend, FrontendSnafu, Record, RequestSnafu, ResponseSnafu, Result};

use super::models::{APIError, DNSRecord, PaginatedResponse, WriteResponse, Zone};

const API_BASE_URL: &str = "https://api.cloudflare.com/client/v4";

enum WriteMethod {
    Create,
    Update,
}

fn process_errors(success: bool, errors: Vec<APIError>) -> Result<()> {
    if !success || errors.len() > 0 {
        let mut err_msg: String = String::new();
        for err in errors {
            err_msg.push_str(&format!("{} {}; ", err.code, err.message));
        }
        return ResponseSnafu {
            message: &format!("Request unsuccessful: {err_msg}"),
        }
        .fail();
    }
    Ok(())
}

pub struct CloudflareFrontend {
    api_key: String,
    domain: url::Host,
    zone_id: Option<String>,
    dns_records: Vec<DNSRecord>,
}

impl CloudflareFrontend {
    fn api_get_paginated<T: DeserializeOwned>(&self, url: &str, per_page: usize) -> Result<Vec<T>> {
        let mut page = 1;
        let mut items: Vec<T> = Vec::new();
        loop {
            let mut resp: PaginatedResponse<T> = ureq::get(url)
                .query("page", &page.to_string())
                .query("per_page", &per_page.to_string())
                .set("X-Auth-Key", &self.api_key)
                .call()
                .context(RequestSnafu { url, method: "GET" })?
                .into_json()
                .boxed_local()
                .context(FrontendSnafu {
                    message: "Failed to deserialize response",
                })?;

            process_errors(resp.success, resp.errors)?;

            items.append(&mut resp.result);

            if let Some(info) = resp.result_info {
                if info.count > info.page * info.per_page {
                    page += 1;
                    continue;
                }
            }

            return Ok(items);
        }
    }

    fn api_write<T: DeserializeOwned>(
        &self,
        url: &str,
        method: WriteMethod,
        body: impl serde::Serialize,
    ) -> Result<T> {
        let req = match method {
            WriteMethod::Create => ureq::post(url),
            WriteMethod::Update => ureq::put(url),
        };
        let resp: WriteResponse<T> = req
            .set("X-Auth-Key", &self.api_key)
            .send_json(body)
            .context(RequestSnafu {
                url,
                method: "POST",
            })?
            .into_json()
            .boxed_local()
            .context(FrontendSnafu {
                message: "Failed to deserialize response",
            })?;

        process_errors(resp.success, resp.errors)?;

        Ok(resp.result)
    }

    fn get_zone_id(&mut self) -> Result<String> {
        if let Some(zone_id) = &self.zone_id {
            return Ok(zone_id.clone());
        }

        let url = format!("{API_BASE_URL}/zones");

        let response: Vec<Zone> = self.api_get_paginated(&url, 50)?;

        for zone in response {
            if zone.name == self.domain.to_string() {
                self.zone_id = Some(zone.id.clone());
                return Ok(zone.id);
            }
        }

        ResponseSnafu {
            message: format!("Failed to find a zone ID for domain {}", self.domain),
        }
        .fail()
    }
}

impl Frontend for CloudflareFrontend {
    fn get_domain(&self) -> url::Host {
        return self.domain.to_owned();
    }

    fn read_records(&mut self) -> Result<Vec<Record>> {
        let zone_id = self.get_zone_id()?;
        let url = format!("{API_BASE_URL}/zones/{zone_id}/dns_records");
        self.dns_records = self.api_get_paginated(&url, 1000)?;
        Ok(self
            .dns_records
            .iter()
            .map(|r| Into::<Record>::into(r.clone()))
            .collect())
    }

    fn write_records(&mut self, records: Vec<Record>) -> Result<()> {
        let mut updates: Vec<DNSRecord> = Vec::with_capacity(records.len());
        let mut new: Vec<DNSRecord> = Vec::with_capacity(records.len());

        let mut exists;
        for record in records {
            exists = false;
            for existing_record in self.dns_records.clone() {
                // TODO handle case where multiples of the same record exist
                if existing_record.kind == record.kind && existing_record.name == record.name {
                    exists = true;
                    // Check if we are attempting to override an unmanaged record
                    if !existing_record.is_managed() {
                        tracing::warn!(
                            kind = existing_record.kind,
                            name = existing_record.name.to_string(),
                            record_id = existing_record.id,
                            "Attempt to override unmanaged record",
                        )
                    }
                    // Only update if content differs
                    else if existing_record.content != record.content {
                        let mut new_record = existing_record.clone();
                        new_record.content = record.content.clone();
                        updates.push(new_record);
                    }
                }
            }

            if !exists {
                new.push(record.into());
            }
        }

        // Write each collection of records
        let zone_id = self.get_zone_id()?;
        for record in new {
            let resp: DNSRecord = self.api_write(
                &format!("{API_BASE_URL}/zones/{zone_id}/dns_records"),
                WriteMethod::Create,
                record,
            )?;
            tracing::info!(
                kind = resp.kind,
                name = resp.name.to_string(),
                record_id = resp.id,
                "Created new record",
            )
        }

        for record in updates {
            let resp: DNSRecord = self.api_write(
                &format!("{API_BASE_URL}/zones/{zone_id}/dns_records/{}", record.id),
                WriteMethod::Update,
                record,
            )?;
            tracing::info!(
                kind = resp.kind,
                name = resp.name.to_string(),
                record_id = resp.id,
                "Updated record",
            )
        }

        Ok(())
    }

    fn delete_records(&mut self, records: Vec<Record>) -> crate::common::Result<()> {
        let zone_id = self.get_zone_id()?;
        for record in records {
            // As a precaution, only delete at most one matching record per execution
            match self
                .dns_records
                .iter()
                .find(|r| r.is_managed() && r.kind == record.kind && r.name == record.name)
            {
                Some(existing_record) => {
                    tracing::info!(
                        kind = existing_record.kind,
                        name = existing_record.name.to_string(),
                        record_id = existing_record.id,
                        "Deleting record",
                    );

                    let url = &format!(
                        "{API_BASE_URL}/zones/{zone_id}/dns_records/{}",
                        existing_record.id,
                    );
                    let status = ureq::delete(url)
                        .set("X-Auth-Key", &self.api_key)
                        .call()
                        .context(RequestSnafu {
                            url,
                            method: "POST",
                        })?
                        .status();

                    if status > 299 {
                        tracing::info!(
                            kind = existing_record.kind,
                            name = existing_record.name.to_string(),
                            record_id = existing_record.id,
                            status = status,
                            "Non-200 response code",
                        );
                    }
                }
                None => {
                    tracing::error!(
                        kind = record.kind,
                        name = record.name.to_string(),
                        "Attempt to delete non-existent record",
                    );
                }
            }
        }
        Ok(())
    }
}

impl From<super::Config> for CloudflareFrontend {
    fn from(value: super::Config) -> Self {
        Self {
            api_key: value.api_key,
            domain: value.domain,
            zone_id: None,
            dns_records: Vec::default(),
        }
    }
}
