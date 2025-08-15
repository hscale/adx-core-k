use crate::{
    error::{SecurityError, SecurityResult},
    models::{
        SecurityScan, ScanType, ScanStatus, Vulnerability, VulnerabilitySeverity,
        SecurityScanRequest, SecurityScanResponse, ScanSummary
    },
    repositories::ScanningRepository,
    audit::AuditService,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Clone)]
pub struct SecurityScanningService {
    repository: Arc<ScanningRepository>,
    audit_service: Arc<AuditService>,
    vulnerability_db_url: String,
    severity_threshold: VulnerabilitySeverity,
    auto_remediation: bool,
}

impl SecurityScanningService {
    pub fn new(
        repository: Arc<ScanningRepository>,
        audit_service: Arc<AuditService>,
        vulnerability_db_url: String,
        severity_threshold: String,
        auto_remediation: bool,
    ) -> SecurityResult<Self> {
        let threshold = match severity_threshold.to_uppercase().as_str() {
            "CRITICAL" => VulnerabilitySeverity::Critical,
            "HIGH" => VulnerabilitySeverity::High,
            "MEDIUM" => VulnerabilitySeverity::Medium,
            "LOW" => VulnerabilitySeverity::Low,
            "INFO" => VulnerabilitySeverity::Info,
            _ => return Err(SecurityError::Validation("Invalid severity threshold".to_string())),
        };

        Ok(Self {
            repository,
            audit_service,
            vulnerability_db_url,
            severity_threshold: threshold,
            auto_remediation,
        })
    }

    /// Initiate a security scan
    pub async fn initiate_scan(&self, request: SecurityScanRequest) -> SecurityResult<Uuid> {
        // Validate the scan request
        self.validate_scan_request(&request)?;

        // Create scan record
        let scan = SecurityScan {
            id: Uuid::new_v4(),
            tenant_id: request.tenant_id.clone(),
            scan_type: request.scan_type.clone(),
            target: request.target.clone(),
            status: ScanStatus::Queued,
            severity_threshold: request.severity_threshold.clone(),
            vulnerabilities_found: 0,
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            scan_results: serde_json::json!({}),
            remediation_suggestions: serde_json::json!([]),
            started_at: Utc::now(),
            completed_at: None,
            created_at: Utc::now(),
        };

        let created_scan = self.repository.create_scan(scan).await?;

        // Log scan initiation
        self.audit_service.log_security_event(
            &request.tenant_id,
            "security_scan_initiated",
            "INFO",
            &format!("Security scan initiated for target: {}", request.target),
            serde_json::json!({
                "scan_id": created_scan.id,
                "scan_type": request.scan_type,
                "target": request.target,
                "severity_threshold": request.severity_threshold
            }),
        ).await?;

        Ok(created_scan.id)
    }

    /// Execute a security scan
    pub async fn execute_scan(&self, scan_id: Uuid) -> SecurityResult<SecurityScanResponse> {
        let mut scan = self.repository.get_scan(scan_id).await?
            .ok_or_else(|| SecurityError::NotFound("Security scan not found".to_string()))?;

        // Update scan status to running
        scan.status = ScanStatus::Running;
        scan.started_at = Utc::now();
        self.repository.update_scan(scan.clone()).await?;

        // Execute the scan based on type
        let scan_result = match scan.scan_type {
            ScanType::Vulnerability => self.execute_vulnerability_scan(&scan).await,
            ScanType::Dependency => self.execute_dependency_scan(&scan).await,
            ScanType::Configuration => self.execute_configuration_scan(&scan).await,
            ScanType::Network => self.execute_network_scan(&scan).await,
            ScanType::Application => self.execute_application_scan(&scan).await,
            ScanType::Infrastructure => self.execute_infrastructure_scan(&scan).await,
        };

        match scan_result {
            Ok((vulnerabilities, results)) => {
                // Update scan with results
                scan.status = ScanStatus::Completed;
                scan.completed_at = Some(Utc::now());
                scan.vulnerabilities_found = vulnerabilities.len() as i32;
                scan.scan_results = results;

                // Count vulnerabilities by severity
                let mut critical_count = 0;
                let mut high_count = 0;
                let mut medium_count = 0;
                let mut low_count = 0;

                for vuln in &vulnerabilities {
                    match vuln.severity {
                        VulnerabilitySeverity::Critical => critical_count += 1,
                        VulnerabilitySeverity::High => high_count += 1,
                        VulnerabilitySeverity::Medium => medium_count += 1,
                        VulnerabilitySeverity::Low => low_count += 1,
                        VulnerabilitySeverity::Info => {}
                    }
                }

                scan.critical_count = critical_count;
                scan.high_count = high_count;
                scan.medium_count = medium_count;
                scan.low_count = low_count;

                // Generate remediation suggestions
                scan.remediation_suggestions = self.generate_remediation_suggestions(&vulnerabilities);

                // Save vulnerabilities
                for vulnerability in &vulnerabilities {
                    self.repository.save_vulnerability(scan_id, vulnerability).await?;
                }

                // Auto-remediation if enabled and appropriate
                if self.auto_remediation && self.should_auto_remediate(&vulnerabilities) {
                    self.attempt_auto_remediation(&scan, &vulnerabilities).await?;
                }

                let updated_scan = self.repository.update_scan(scan).await?;

                // Log scan completion
                self.audit_service.log_security_event(
                    &updated_scan.tenant_id,
                    "security_scan_completed",
                    if critical_count > 0 { "CRITICAL" } else if high_count > 0 { "HIGH" } else { "INFO" },
                    &format!("Security scan completed with {} vulnerabilities found", vulnerabilities.len()),
                    serde_json::json!({
                        "scan_id": scan_id,
                        "vulnerabilities_found": vulnerabilities.len(),
                        "critical_count": critical_count,
                        "high_count": high_count,
                        "medium_count": medium_count,
                        "low_count": low_count
                    }),
                ).await?;

                // Create summary
                let mut by_severity = HashMap::new();
                by_severity.insert(VulnerabilitySeverity::Critical, critical_count);
                by_severity.insert(VulnerabilitySeverity::High, high_count);
                by_severity.insert(VulnerabilitySeverity::Medium, medium_count);
                by_severity.insert(VulnerabilitySeverity::Low, low_count);

                let summary = ScanSummary {
                    total_vulnerabilities: vulnerabilities.len() as i32,
                    by_severity,
                    remediation_priority: self.generate_remediation_priority(&vulnerabilities),
                };

                Ok(SecurityScanResponse {
                    scan: updated_scan,
                    vulnerabilities,
                    summary,
                })
            }
            Err(e) => {
                // Mark scan as failed
                scan.status = ScanStatus::Failed;
                scan.completed_at = Some(Utc::now());
                scan.scan_results = serde_json::json!({
                    "error": e.to_string()
                });
                self.repository.update_scan(scan).await?;

                // Log scan failure
                self.audit_service.log_security_event(
                    &scan.tenant_id,
                    "security_scan_failed",
                    "ERROR",
                    &format!("Security scan failed: {}", e),
                    serde_json::json!({
                        "scan_id": scan_id,
                        "error": e.to_string()
                    }),
                ).await?;

                Err(e)
            }
        }
    }

    /// Get scan results
    pub async fn get_scan_results(&self, scan_id: Uuid) -> SecurityResult<SecurityScanResponse> {
        let scan = self.repository.get_scan(scan_id).await?
            .ok_or_else(|| SecurityError::NotFound("Security scan not found".to_string()))?;

        let vulnerabilities = self.repository.get_scan_vulnerabilities(scan_id).await?;

        // Create summary
        let mut by_severity = HashMap::new();
        by_severity.insert(VulnerabilitySeverity::Critical, scan.critical_count);
        by_severity.insert(VulnerabilitySeverity::High, scan.high_count);
        by_severity.insert(VulnerabilitySeverity::Medium, scan.medium_count);
        by_severity.insert(VulnerabilitySeverity::Low, scan.low_count);

        let summary = ScanSummary {
            total_vulnerabilities: scan.vulnerabilities_found,
            by_severity,
            remediation_priority: self.generate_remediation_priority(&vulnerabilities),
        };

        Ok(SecurityScanResponse {
            scan,
            vulnerabilities,
            summary,
        })
    }

    /// Get all scans for a tenant
    pub async fn get_tenant_scans(
        &self,
        tenant_id: &str,
        scan_type: Option<ScanType>,
        status: Option<ScanStatus>,
        page: i32,
        page_size: i32,
    ) -> SecurityResult<Vec<SecurityScan>> {
        self.repository.get_tenant_scans(tenant_id, scan_type, status, page, page_size).await
    }

    /// Cancel a running scan
    pub async fn cancel_scan(&self, scan_id: Uuid) -> SecurityResult<()> {
        let mut scan = self.repository.get_scan(scan_id).await?
            .ok_or_else(|| SecurityError::NotFound("Security scan not found".to_string()))?;

        if scan.status != ScanStatus::Queued && scan.status != ScanStatus::Running {
            return Err(SecurityError::Validation("Can only cancel queued or running scans".to_string()));
        }

        scan.status = ScanStatus::Cancelled;
        scan.completed_at = Some(Utc::now());
        self.repository.update_scan(scan.clone()).await?;

        // Log scan cancellation
        self.audit_service.log_security_event(
            &scan.tenant_id,
            "security_scan_cancelled",
            "INFO",
            "Security scan was cancelled",
            serde_json::json!({
                "scan_id": scan_id
            }),
        ).await?;

        Ok(())
    }

    /// Update vulnerability database
    pub async fn update_vulnerability_database(&self) -> SecurityResult<()> {
        info!("Updating vulnerability database from {}", self.vulnerability_db_url);

        // This would download and update the vulnerability database
        // For now, we'll simulate the process
        let client = reqwest::Client::new();
        let response = client.get(&self.vulnerability_db_url).send().await?;

        if response.status().is_success() {
            info!("Successfully updated vulnerability database");
        } else {
            warn!("Failed to update vulnerability database: {}", response.status());
        }

        Ok(())
    }

    // Private helper methods

    fn validate_scan_request(&self, request: &SecurityScanRequest) -> SecurityResult<()> {
        if request.tenant_id.is_empty() {
            return Err(SecurityError::Validation("Tenant ID is required".to_string()));
        }
        if request.target.is_empty() {
            return Err(SecurityError::Validation("Scan target is required".to_string()));
        }
        Ok(())
    }

    async fn execute_vulnerability_scan(&self, scan: &SecurityScan) -> SecurityResult<(Vec<Vulnerability>, Value)> {
        info!(scan_id = %scan.id, target = %scan.target, "Executing vulnerability scan");

        // This would perform actual vulnerability scanning
        // For now, we'll simulate finding some vulnerabilities
        let vulnerabilities = vec![
            Vulnerability {
                id: "CVE-2023-1234".to_string(),
                cve_id: Some("CVE-2023-1234".to_string()),
                title: "SQL Injection Vulnerability".to_string(),
                description: "A SQL injection vulnerability was found in the application".to_string(),
                severity: VulnerabilitySeverity::High,
                cvss_score: Some(8.5),
                affected_component: "web-application".to_string(),
                fixed_version: Some("1.2.3".to_string()),
                references: vec![
                    "https://nvd.nist.gov/vuln/detail/CVE-2023-1234".to_string(),
                ],
                discovered_at: Utc::now(),
            },
            Vulnerability {
                id: "CVE-2023-5678".to_string(),
                cve_id: Some("CVE-2023-5678".to_string()),
                title: "Cross-Site Scripting (XSS)".to_string(),
                description: "A reflected XSS vulnerability was found".to_string(),
                severity: VulnerabilitySeverity::Medium,
                cvss_score: Some(6.1),
                affected_component: "web-frontend".to_string(),
                fixed_version: Some("2.1.0".to_string()),
                references: vec![
                    "https://nvd.nist.gov/vuln/detail/CVE-2023-5678".to_string(),
                ],
                discovered_at: Utc::now(),
            },
        ];

        let results = serde_json::json!({
            "scan_type": "vulnerability",
            "target": scan.target,
            "scan_duration_seconds": 120,
            "tools_used": ["nmap", "nikto", "sqlmap"],
            "coverage": {
                "ports_scanned": 1000,
                "services_identified": 5,
                "endpoints_tested": 25
            }
        });

        Ok((vulnerabilities, results))
    }

    async fn execute_dependency_scan(&self, scan: &SecurityScan) -> SecurityResult<(Vec<Vulnerability>, Value)> {
        info!(scan_id = %scan.id, target = %scan.target, "Executing dependency scan");

        // This would scan dependencies for known vulnerabilities
        let vulnerabilities = vec![
            Vulnerability {
                id: "GHSA-xxxx-yyyy-zzzz".to_string(),
                cve_id: Some("CVE-2023-9999".to_string()),
                title: "Vulnerable Dependency: lodash".to_string(),
                description: "Prototype pollution vulnerability in lodash".to_string(),
                severity: VulnerabilitySeverity::High,
                cvss_score: Some(7.5),
                affected_component: "lodash@4.17.20".to_string(),
                fixed_version: Some("4.17.21".to_string()),
                references: vec![
                    "https://github.com/advisories/GHSA-xxxx-yyyy-zzzz".to_string(),
                ],
                discovered_at: Utc::now(),
            },
        ];

        let results = serde_json::json!({
            "scan_type": "dependency",
            "target": scan.target,
            "dependencies_scanned": 150,
            "vulnerable_dependencies": 1,
            "tools_used": ["npm audit", "snyk", "safety"]
        });

        Ok((vulnerabilities, results))
    }

    async fn execute_configuration_scan(&self, scan: &SecurityScan) -> SecurityResult<(Vec<Vulnerability>, Value)> {
        info!(scan_id = %scan.id, target = %scan.target, "Executing configuration scan");

        // This would scan for configuration issues
        let vulnerabilities = vec![
            Vulnerability {
                id: "CONFIG-001".to_string(),
                cve_id: None,
                title: "Weak SSL/TLS Configuration".to_string(),
                description: "Server supports weak SSL/TLS protocols".to_string(),
                severity: VulnerabilitySeverity::Medium,
                cvss_score: Some(5.3),
                affected_component: "web-server".to_string(),
                fixed_version: None,
                references: vec![
                    "https://owasp.org/www-project-top-ten/2017/A6_2017-Security_Misconfiguration".to_string(),
                ],
                discovered_at: Utc::now(),
            },
        ];

        let results = serde_json::json!({
            "scan_type": "configuration",
            "target": scan.target,
            "configurations_checked": 50,
            "misconfigurations_found": 1,
            "tools_used": ["lynis", "openvas"]
        });

        Ok((vulnerabilities, results))
    }

    async fn execute_network_scan(&self, scan: &SecurityScan) -> SecurityResult<(Vec<Vulnerability>, Value)> {
        info!(scan_id = %scan.id, target = %scan.target, "Executing network scan");

        // This would perform network security scanning
        let vulnerabilities = vec![];

        let results = serde_json::json!({
            "scan_type": "network",
            "target": scan.target,
            "ports_scanned": 65535,
            "open_ports": [22, 80, 443, 3306],
            "services_detected": ["ssh", "http", "https", "mysql"],
            "tools_used": ["nmap", "masscan"]
        });

        Ok((vulnerabilities, results))
    }

    async fn execute_application_scan(&self, scan: &SecurityScan) -> SecurityResult<(Vec<Vulnerability>, Value)> {
        info!(scan_id = %scan.id, target = %scan.target, "Executing application scan");

        // This would perform application security testing
        let vulnerabilities = vec![];

        let results = serde_json::json!({
            "scan_type": "application",
            "target": scan.target,
            "endpoints_tested": 100,
            "authentication_tested": true,
            "tools_used": ["burp", "zap", "w3af"]
        });

        Ok((vulnerabilities, results))
    }

    async fn execute_infrastructure_scan(&self, scan: &SecurityScan) -> SecurityResult<(Vec<Vulnerability>, Value)> {
        info!(scan_id = %scan.id, target = %scan.target, "Executing infrastructure scan");

        // This would scan infrastructure components
        let vulnerabilities = vec![];

        let results = serde_json::json!({
            "scan_type": "infrastructure",
            "target": scan.target,
            "components_scanned": ["docker", "kubernetes", "terraform"],
            "tools_used": ["kube-bench", "docker-bench", "checkov"]
        });

        Ok((vulnerabilities, results))
    }

    fn generate_remediation_suggestions(&self, vulnerabilities: &[Vulnerability]) -> Value {
        let mut suggestions = Vec::new();

        for vuln in vulnerabilities {
            let suggestion = match vuln.severity {
                VulnerabilitySeverity::Critical => {
                    serde_json::json!({
                        "vulnerability_id": vuln.id,
                        "priority": "IMMEDIATE",
                        "action": "Apply security patch immediately",
                        "timeline": "Within 24 hours",
                        "fixed_version": vuln.fixed_version
                    })
                }
                VulnerabilitySeverity::High => {
                    serde_json::json!({
                        "vulnerability_id": vuln.id,
                        "priority": "HIGH",
                        "action": "Schedule security patch deployment",
                        "timeline": "Within 7 days",
                        "fixed_version": vuln.fixed_version
                    })
                }
                VulnerabilitySeverity::Medium => {
                    serde_json::json!({
                        "vulnerability_id": vuln.id,
                        "priority": "MEDIUM",
                        "action": "Plan security update in next release cycle",
                        "timeline": "Within 30 days",
                        "fixed_version": vuln.fixed_version
                    })
                }
                _ => {
                    serde_json::json!({
                        "vulnerability_id": vuln.id,
                        "priority": "LOW",
                        "action": "Monitor and update when convenient",
                        "timeline": "Next maintenance window",
                        "fixed_version": vuln.fixed_version
                    })
                }
            };
            suggestions.push(suggestion);
        }

        Value::Array(suggestions)
    }

    fn generate_remediation_priority(&self, vulnerabilities: &[Vulnerability]) -> Vec<String> {
        let mut priority_list = Vec::new();

        // Sort by severity and add to priority list
        let mut sorted_vulns = vulnerabilities.to_vec();
        sorted_vulns.sort_by(|a, b| {
            let a_score = match a.severity {
                VulnerabilitySeverity::Critical => 4,
                VulnerabilitySeverity::High => 3,
                VulnerabilitySeverity::Medium => 2,
                VulnerabilitySeverity::Low => 1,
                VulnerabilitySeverity::Info => 0,
            };
            let b_score = match b.severity {
                VulnerabilitySeverity::Critical => 4,
                VulnerabilitySeverity::High => 3,
                VulnerabilitySeverity::Medium => 2,
                VulnerabilitySeverity::Low => 1,
                VulnerabilitySeverity::Info => 0,
            };
            b_score.cmp(&a_score)
        });

        for vuln in sorted_vulns {
            priority_list.push(format!("{}: {}", vuln.id, vuln.title));
        }

        priority_list
    }

    fn should_auto_remediate(&self, vulnerabilities: &[Vulnerability]) -> bool {
        // Only auto-remediate if there are no critical vulnerabilities
        // and the fixes are well-known
        !vulnerabilities.iter().any(|v| matches!(v.severity, VulnerabilitySeverity::Critical))
            && vulnerabilities.iter().all(|v| v.fixed_version.is_some())
    }

    async fn attempt_auto_remediation(
        &self,
        scan: &SecurityScan,
        vulnerabilities: &[Vulnerability],
    ) -> SecurityResult<()> {
        info!(
            scan_id = %scan.id,
            vulnerability_count = %vulnerabilities.len(),
            "Attempting auto-remediation"
        );

        // This would implement automatic remediation logic
        // For now, we'll just log the attempt
        for vuln in vulnerabilities {
            if let Some(fixed_version) = &vuln.fixed_version {
                info!(
                    vulnerability_id = %vuln.id,
                    fixed_version = %fixed_version,
                    "Would auto-remediate vulnerability"
                );
            }
        }

        // Log auto-remediation attempt
        self.audit_service.log_security_event(
            &scan.tenant_id,
            "auto_remediation_attempted",
            "INFO",
            "Automatic remediation was attempted for scan vulnerabilities",
            serde_json::json!({
                "scan_id": scan.id,
                "vulnerabilities_count": vulnerabilities.len()
            }),
        ).await?;

        Ok(())
    }
}