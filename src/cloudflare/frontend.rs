use std::time::Duration;

use serde::de::DeserializeOwned;
use snafu::prelude::*;

use crate::common::{
    self, diff_records, key_file_or_string, Error, FrontendSnafu, Record, RequestSnafu,
    ResponseSnafu, Result,
};

use super::models::{APIError, DNSRecord, DeleteResponse, PaginatedResponse, WriteResponse, Zone};

const API_BASE_URL: &str = "https://api.cloudflare.com/client/v4";

pub const FRONTEND_NAME: &str = "Cloudflare";

enum WriteMethod {
    Create,
    Delete,
    Update,
}

impl ToString for WriteMethod {
    fn to_string(&self) -> String {
        match self {
            WriteMethod::Create => "Create",
            WriteMethod::Delete => "Delete",
            WriteMethod::Update => "Update",
        }
        .to_string()
    }
}

impl From<WriteMethod> for String {
    fn from(value: WriteMethod) -> Self {
        value.to_string()
    }
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

pub struct Cloudflare {
    agent: ureq::Agent,
    api_key: String,
    domain: String,
    instance_id: String,
    zone_id: Option<String>,
}

impl Cloudflare {
    fn with_headers<T>(&self, req: ureq::RequestBuilder<T>) -> ureq::RequestBuilder<T> {
        req.header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json; charset=utf8")
    }

    fn api_get_paginated<T: DeserializeOwned>(&self, url: &str, per_page: usize) -> Result<Vec<T>> {
        let mut page = 1;
        let mut items: Vec<T> = Vec::new();
        loop {
            tracing::debug!(
                url = url,
                method = "GET",
                frontend = "cloudflare",
                "Sending request"
            );
            let mut res = self
                .with_headers(self.agent.get(url))
                .query("page", &page.to_string())
                .query("per_page", &per_page.to_string())
                .call()
                .context(RequestSnafu {
                    url,
                    method: "Read",
                })?;

            let mut resp: PaginatedResponse<T> =
                res.body_mut()
                    .read_json()
                    .boxed_local()
                    .context(FrontendSnafu {
                        frontend: FRONTEND_NAME,
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
        let mut res = match method {
            WriteMethod::Create => self.with_headers(self.agent.post(url)).send_json(body),
            WriteMethod::Delete => self.with_headers(self.agent.delete(url)).call(),
            WriteMethod::Update => self.with_headers(self.agent.put(url)).send_json(body),
        }
        .context(RequestSnafu {
            url,
            method: method.to_string(),
        })?;

        if !res.status().is_success() {
            let response_body = res
                .body_mut()
                .read_to_string()
                .unwrap_or_else(|_| "No body".to_string());

            return Err(Error::ResponseError {
                message: format!("{} {} failed: {}", method.to_string(), url, response_body),
            });
        }

        let resp: WriteResponse<T> =
            res.body_mut()
                .read_json()
                .boxed_local()
                .context(FrontendSnafu {
                    frontend: FRONTEND_NAME,
                    message: "Failed to deserialize response",
                })?;

        process_errors(resp.success, resp.errors)?;

        Ok(resp.result)
    }

    pub(super) fn get_zone_id(&mut self) -> Result<String> {
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

    pub(super) fn read_records(&self, zone_id: String) -> Result<Vec<DNSRecord>> {
        let url = format!("{API_BASE_URL}/zones/{zone_id}/dns_records");
        let records: Vec<DNSRecord> = self.api_get_paginated(&url, 1000)?;
        Ok(records.into_iter().collect())
    }
}

impl common::Frontend for Cloudflare {
    fn get_domain(&self) -> &str {
        return &self.domain;
    }

    fn set_records(&mut self, authority: Vec<Record>, dry_run: bool) -> Result<()> {
        let zone_id = self.get_zone_id()?;
        let current = self.read_records(zone_id.clone())?;
        let diff = diff_records::<DNSRecord>(current, authority);

        // Evaluate deletions more precisely based on instance ID
        let diff_len = diff.len() - diff.delete.len();
        let deletions: Vec<DNSRecord> = diff
            .delete
            .into_iter()
            .filter(|record| record.get_instance_id().eq(&Some(&self.instance_id)))
            .collect();
        let diff_len = diff_len + deletions.len();

        // Short circuit on no changes
        if diff_len == 0 {
            tracing::info!(frontend = FRONTEND_NAME, "No changes detected",);
            return Ok(());
        }

        // Stop on dry run
        if dry_run {
            tracing::info!(
                frontend = FRONTEND_NAME,
                create = diff.create.len(),
                update = diff.update.len(),
                delete = deletions.len(),
                "Dry run completed",
            );
            return Ok(());
        }

        tracing::info!(
            frontend = FRONTEND_NAME,
            create = diff.create.len(),
            update = diff.update.len(),
            delete = deletions.len(),
            "Applying changes",
        );

        // Write each collection of records.
        // Deletes first - to avoid any key/unique errors.
        for record in deletions {
            tracing::info!(
                frontend = FRONTEND_NAME,
                kind = record.kind,
                name = record.name,
                record_id = record.id,
                "Deleting record",
            );

            self.api_write::<DeleteResponse>(
                &format!("{API_BASE_URL}/zones/{zone_id}/dns_records/{}", record.id),
                WriteMethod::Delete,
                record.clone(),
            )?;

            tracing::debug!(
                frontend = FRONTEND_NAME,
                kind = record.kind,
                name = record.name,
                record_id = record.id,
                "Deleted record",
            );
        }

        for mut record in diff.update {
            tracing::info!(
                frontend = FRONTEND_NAME,
                name = record.name,
                kind = record.kind,
                content = record.content,
                record_id = record.id,
                "Updating record"
            );

            if record.get_instance_id().ne(&Some(&self.instance_id)) {
                tracing::warn!(
                    frontend = FRONTEND_NAME,
                    name = record.name,
                    kind = record.kind,
                    content = record.content,
                    record_id = record.id,
                    "Record is not managed by this instance but we will update anyway"
                )
            }
            record.set_instance_id(&self.instance_id);

            let resp: DNSRecord = self.api_write(
                &format!("{API_BASE_URL}/zones/{zone_id}/dns_records/{}", record.id),
                WriteMethod::Update,
                record,
            )?;

            tracing::debug!(
                frontend = FRONTEND_NAME,
                kind = resp.kind,
                name = resp.name,
                record_id = resp.id,
                "Updated record",
            );
        }

        for mut record in diff.create {
            tracing::info!(
                frontend = FRONTEND_NAME,
                name = record.name,
                kind = record.kind,
                content = record.content,
                "Creating record"
            );
            record.set_instance_id(&self.instance_id);

            let resp: DNSRecord = self.api_write(
                &format!("{API_BASE_URL}/zones/{zone_id}/dns_records"),
                WriteMethod::Create,
                record,
            )?;

            tracing::debug!(
                frontend = FRONTEND_NAME,
                kind = resp.kind,
                name = resp.name,
                record_id = resp.id,
                "Created record",
            );
        }

        Ok(())
    }
}

impl From<super::Config> for Cloudflare {
    fn from(value: super::Config) -> Self {
        let api_key = key_file_or_string(value.api_key, FRONTEND_NAME.into()).unwrap();

        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .build();

        Self {
            agent: config.into(),
            api_key,
            domain: value.domain,
            instance_id: value.instance_id,
            zone_id: None,
        }
    }
}
