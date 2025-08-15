use crate::config::WhiteLabelConfig;
use crate::error::{WhiteLabelError, WhiteLabelResult};
use crate::types::{DnsRecord, DomainVerificationResult};
use std::sync::Arc;
use trust_dns_resolver::{Resolver, config::*};

pub struct DnsService {
    config: Arc<WhiteLabelConfig>,
    resolver: Resolver,
}

impl DnsService {
    pub fn new(config: Arc<WhiteLabelConfig>) -> WhiteLabelResult<Self> {
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())
            .map_err(|e| WhiteLabelError::Configuration(format!("DNS resolver error: {}", e)))?;

        Ok(Self { config, resolver })
    }

    pub async fn verify_records(
        &self,
        domain: &str,
        expected_records: &[DnsRecord],
    ) -> WhiteLabelResult<DomainVerificationResult> {
        let mut dns_records_found = Vec::new();
        let mut all_verified = true;
        let mut error_messages = Vec::new();

        for expected_record in expected_records {
            match self.verify_single_record(domain, expected_record).await {
                Ok(found_record) => {
                    dns_records_found.push(found_record);
                }
                Err(e) => {
                    all_verified = false;
                    error_messages.push(format!(
                        "Failed to verify {} record for {}: {}",
                        expected_record.record_type, expected_record.name, e
                    ));
                }
            }
        }

        Ok(DomainVerificationResult {
            verified: all_verified,
            verification_method: "DNS".to_string(),
            dns_records_found,
            error_message: if error_messages.is_empty() {
                None
            } else {
                Some(error_messages.join("; "))
            },
        })
    }

    async fn verify_single_record(
        &self,
        _domain: &str,
        expected_record: &DnsRecord,
    ) -> WhiteLabelResult<DnsRecord> {
        match expected_record.record_type.as_str() {
            "TXT" => self.verify_txt_record(expected_record).await,
            "CNAME" => self.verify_cname_record(expected_record).await,
            "A" => self.verify_a_record(expected_record).await,
            _ => Err(WhiteLabelError::DnsVerification(format!(
                "Unsupported record type: {}",
                expected_record.record_type
            ))),
        }
    }

    async fn verify_txt_record(&self, expected_record: &DnsRecord) -> WhiteLabelResult<DnsRecord> {
        use trust_dns_resolver::rr::RecordType;

        let response = self
            .resolver
            .lookup(&expected_record.name, RecordType::TXT)
            .map_err(|e| {
                WhiteLabelError::DnsVerification(format!("TXT lookup failed: {}", e))
            })?;

        for record in response.iter() {
            if let Some(txt_data) = record.as_txt() {
                let txt_value = txt_data
                    .txt_data()
                    .iter()
                    .map(|bytes| String::from_utf8_lossy(bytes))
                    .collect::<Vec<_>>()
                    .join("");

                if txt_value.contains(&expected_record.value) {
                    return Ok(DnsRecord {
                        record_type: "TXT".to_string(),
                        name: expected_record.name.clone(),
                        value: txt_value,
                        ttl: record.ttl(),
                    });
                }
            }
        }

        Err(WhiteLabelError::DnsVerification(format!(
            "TXT record not found or value mismatch for {}",
            expected_record.name
        )))
    }

    async fn verify_cname_record(&self, expected_record: &DnsRecord) -> WhiteLabelResult<DnsRecord> {
        use trust_dns_resolver::rr::RecordType;

        let response = self
            .resolver
            .lookup(&expected_record.name, RecordType::CNAME)
            .map_err(|e| {
                WhiteLabelError::DnsVerification(format!("CNAME lookup failed: {}", e))
            })?;

        for record in response.iter() {
            if let Some(cname_data) = record.as_cname() {
                let cname_value = cname_data.to_string();
                
                // Remove trailing dot if present
                let cname_value = cname_value.trim_end_matches('.');
                let expected_value = expected_record.value.trim_end_matches('.');

                if cname_value == expected_value {
                    return Ok(DnsRecord {
                        record_type: "CNAME".to_string(),
                        name: expected_record.name.clone(),
                        value: cname_value.to_string(),
                        ttl: record.ttl(),
                    });
                }
            }
        }

        Err(WhiteLabelError::DnsVerification(format!(
            "CNAME record not found or value mismatch for {}",
            expected_record.name
        )))
    }

    async fn verify_a_record(&self, expected_record: &DnsRecord) -> WhiteLabelResult<DnsRecord> {
        use trust_dns_resolver::rr::RecordType;

        let response = self
            .resolver
            .lookup(&expected_record.name, RecordType::A)
            .map_err(|e| {
                WhiteLabelError::DnsVerification(format!("A record lookup failed: {}", e))
            })?;

        for record in response.iter() {
            if let Some(a_data) = record.as_a() {
                let ip_value = a_data.to_string();

                if ip_value == expected_record.value {
                    return Ok(DnsRecord {
                        record_type: "A".to_string(),
                        name: expected_record.name.clone(),
                        value: ip_value,
                        ttl: record.ttl(),
                    });
                }
            }
        }

        Err(WhiteLabelError::DnsVerification(format!(
            "A record not found or value mismatch for {}",
            expected_record.name
        )))
    }

    pub async fn create_dns_records(
        &self,
        domain: &str,
        records: &[DnsRecord],
        provider: &str,
    ) -> WhiteLabelResult<()> {
        // This would integrate with DNS providers like Cloudflare, Route53, etc.
        tracing::info!(
            "Creating DNS records for domain {} using provider {}",
            domain,
            provider
        );

        for record in records {
            tracing::info!(
                "Creating {} record: {} -> {}",
                record.record_type,
                record.name,
                record.value
            );
        }

        // In a real implementation, this would make API calls to the DNS provider
        Ok(())
    }

    pub async fn delete_dns_records(
        &self,
        domain: &str,
        records: &[DnsRecord],
        provider: &str,
    ) -> WhiteLabelResult<()> {
        tracing::info!(
            "Deleting DNS records for domain {} using provider {}",
            domain,
            provider
        );

        for record in records {
            tracing::info!(
                "Deleting {} record: {}",
                record.record_type,
                record.name
            );
        }

        Ok(())
    }
}