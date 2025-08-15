use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use tracing::{info, warn, error};

use crate::config::ModuleServiceConfig;
use crate::error::ModuleServiceError;
use crate::types::{SecurityScanResults, SecurityVulnerability, VulnerabilitySeverity};

/// Comprehensive security scanner for modules
#[async_trait]
pub trait SecurityScannerTrait {
    async fn scan_package(&self, package_path: &str, deep_scan: bool) -> Result<SecurityScanResults, ModuleServiceError>;
    async fn scan_static_analysis(&self, package_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError>;
    async fn scan_dependencies(&self, package_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError>;
    async fn scan_malware(&self, package_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError>;
    async fn scan_license_compliance(&self, package_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError>;
    async fn validate_signatures(&self, package_path: &str) -> Result<bool, ModuleServiceError>;
}

pub struct SecurityScanner {
    config: ModuleServiceConfig,
    scanner_rules: ScannerRules,
}

impl SecurityScanner {
    pub fn new(config: ModuleServiceConfig) -> Self {
        Self {
            config,
            scanner_rules: ScannerRules::default(),
        }
    }
}

#[async_trait]
impl SecurityScannerTrait for SecurityScanner {
    async fn scan_package(&self, package_path: &str, deep_scan: bool) -> Result<SecurityScanResults, ModuleServiceError> {
        let start_time = Instant::now();
        info!("Starting security scan for package: {}", package_path);

        let mut vulnerabilities = Vec::new();
        let mut scan_passed = true;

        // Extract package for scanning
        let temp_dir = self.extract_package_for_scan(package_path).await?;
        
        // Static analysis scan
        match self.scan_static_analysis(&temp_dir).await {
            Ok(mut static_vulns) => vulnerabilities.append(&mut static_vulns),
            Err(e) => {
                error!("Static analysis scan failed: {}", e);
                vulnerabilities.push(SecurityVulnerability {
                    severity: VulnerabilitySeverity::Medium,
                    category: "scan_error".to_string(),
                    description: format!("Static analysis scan failed: {}", e),
                    file_path: None,
                    line_number: None,
                    recommendation: "Manual review required".to_string(),
                });
            }
        }

        // Dependency scan
        match self.scan_dependencies(&temp_dir).await {
            Ok(mut dep_vulns) => vulnerabilities.append(&mut dep_vulns),
            Err(e) => {
                error!("Dependency scan failed: {}", e);
                vulnerabilities.push(SecurityVulnerability {
                    severity: VulnerabilitySeverity::Medium,
                    category: "scan_error".to_string(),
                    description: format!("Dependency scan failed: {}", e),
                    file_path: None,
                    line_number: None,
                    recommendation: "Manual review required".to_string(),
                });
            }
        }

        // Malware scan
        match self.scan_malware(&temp_dir).await {
            Ok(mut malware_vulns) => vulnerabilities.append(&mut malware_vulns),
            Err(e) => {
                error!("Malware scan failed: {}", e);
                vulnerabilities.push(SecurityVulnerability {
                    severity: VulnerabilitySeverity::High,
                    category: "scan_error".to_string(),
                    description: format!("Malware scan failed: {}", e),
                    file_path: None,
                    line_number: None,
                    recommendation: "Manual review required".to_string(),
                });
            }
        }

        // License compliance scan
        if deep_scan {
            match self.scan_license_compliance(&temp_dir).await {
                Ok(mut license_vulns) => vulnerabilities.append(&mut license_vulns),
                Err(e) => {
                    warn!("License compliance scan failed: {}", e);
                    // License issues are warnings, not failures
                }
            }
        }

        // Signature validation
        if self.config.security.signature_verification {
            match self.validate_signatures(package_path).await {
                Ok(valid) => {
                    if !valid {
                        vulnerabilities.push(SecurityVulnerability {
                            severity: VulnerabilitySeverity::High,
                            category: "signature".to_string(),
                            description: "Package signature validation failed".to_string(),
                            file_path: None,
                            line_number: None,
                            recommendation: "Verify package authenticity".to_string(),
                        });
                    }
                }
                Err(e) => {
                    error!("Signature validation failed: {}", e);
                    vulnerabilities.push(SecurityVulnerability {
                        severity: VulnerabilitySeverity::Medium,
                        category: "signature".to_string(),
                        description: format!("Signature validation error: {}", e),
                        file_path: None,
                        line_number: None,
                        recommendation: "Manual signature verification required".to_string(),
                    });
                }
            }
        }

        // Calculate security score
        let score = self.calculate_security_score(&vulnerabilities);
        
        // Determine if scan passed
        let critical_count = vulnerabilities.iter()
            .filter(|v| matches!(v.severity, VulnerabilitySeverity::Critical))
            .count();
        let high_count = vulnerabilities.iter()
            .filter(|v| matches!(v.severity, VulnerabilitySeverity::High))
            .count();

        scan_passed = critical_count == 0 && high_count <= 2 && score >= 70;

        // Cleanup temp directory
        let _ = std::fs::remove_dir_all(&temp_dir);

        let scan_duration = start_time.elapsed();
        
        info!(
            "Security scan completed for {}: passed={}, score={}, vulnerabilities={}, duration={:?}",
            package_path, scan_passed, score, vulnerabilities.len(), scan_duration
        );

        Ok(SecurityScanResults {
            passed: scan_passed,
            score,
            vulnerabilities,
            scan_date: chrono::Utc::now(),
            scanner_version: "1.0.0".to_string(),
        })
    }

    async fn scan_static_analysis(&self, package_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError> {
        let mut vulnerabilities = Vec::new();

        // Scan JavaScript/TypeScript files
        let js_files = self.find_files_by_extension(package_path, &["js", "ts", "jsx", "tsx"]).await?;
        
        for file_path in js_files {
            let file_vulns = self.scan_javascript_file(&file_path).await?;
            vulnerabilities.extend(file_vulns);
        }

        // Scan configuration files
        let config_files = self.find_files_by_name(package_path, &["package.json", "webpack.config.js", "vite.config.js"]).await?;
        
        for file_path in config_files {
            let config_vulns = self.scan_config_file(&file_path).await?;
            vulnerabilities.extend(config_vulns);
        }

        // Scan for sensitive data
        let sensitive_vulns = self.scan_sensitive_data(package_path).await?;
        vulnerabilities.extend(sensitive_vulns);

        Ok(vulnerabilities)
    }

    async fn scan_dependencies(&self, package_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError> {
        let mut vulnerabilities = Vec::new();

        // Check for package.json
        let package_json_path = format!("{}/package.json", package_path);
        if Path::new(&package_json_path).exists() {
            let package_content = std::fs::read_to_string(&package_json_path)
                .map_err(|e| ModuleServiceError::IoError(e))?;
            
            let package_json: serde_json::Value = serde_json::from_str(&package_content)
                .map_err(|e| ModuleServiceError::SerializationError(e))?;

            // Check dependencies
            if let Some(deps) = package_json.get("dependencies").and_then(|d| d.as_object()) {
                for (dep_name, dep_version) in deps {
                    let dep_vulns = self.check_dependency_vulnerabilities(dep_name, dep_version.as_str().unwrap_or("*")).await?;
                    vulnerabilities.extend(dep_vulns);
                }
            }

            // Check devDependencies
            if let Some(dev_deps) = package_json.get("devDependencies").and_then(|d| d.as_object()) {
                for (dep_name, dep_version) in dev_deps {
                    let dep_vulns = self.check_dependency_vulnerabilities(dep_name, dep_version.as_str().unwrap_or("*")).await?;
                    vulnerabilities.extend(dep_vulns);
                }
            }
        }

        // Check for known vulnerable patterns
        let pattern_vulns = self.scan_vulnerable_patterns(package_path).await?;
        vulnerabilities.extend(pattern_vulns);

        Ok(vulnerabilities)
    }

    async fn scan_malware(&self, package_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError> {
        let mut vulnerabilities = Vec::new();

        // Scan for malicious patterns
        let malicious_patterns = vec![
            r"eval\s*\(",
            r"Function\s*\(",
            r"setTimeout\s*\(\s*['\"]",
            r"setInterval\s*\(\s*['\"]",
            r"document\.write\s*\(",
            r"innerHTML\s*=",
            r"crypto\.createHash",
            r"child_process",
            r"fs\.readFile",
            r"fs\.writeFile",
            r"require\s*\(\s*['\"]child_process['\"]",
            r"require\s*\(\s*['\"]fs['\"]",
            r"XMLHttpRequest",
            r"fetch\s*\(",
        ];

        let files = self.find_all_text_files(package_path).await?;
        
        for file_path in files {
            let content = match std::fs::read_to_string(&file_path) {
                Ok(content) => content,
                Err(_) => continue, // Skip files that can't be read
            };

            for pattern in &malicious_patterns {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    for mat in regex.find_iter(&content) {
                        let line_number = content[..mat.start()].matches('\n').count() + 1;
                        
                        vulnerabilities.push(SecurityVulnerability {
                            severity: self.get_pattern_severity(pattern),
                            category: "malicious_pattern".to_string(),
                            description: format!("Potentially malicious pattern found: {}", pattern),
                            file_path: Some(file_path.clone()),
                            line_number: Some(line_number as u32),
                            recommendation: "Review code for malicious intent".to_string(),
                        });
                    }
                }
            }
        }

        // Check file sizes for potential data exfiltration
        let large_files = self.find_large_files(package_path, 10 * 1024 * 1024).await?; // 10MB
        for file_path in large_files {
            vulnerabilities.push(SecurityVulnerability {
                severity: VulnerabilitySeverity::Medium,
                category: "suspicious_file".to_string(),
                description: "Unusually large file detected".to_string(),
                file_path: Some(file_path),
                line_number: None,
                recommendation: "Review file contents for legitimacy".to_string(),
            });
        }

        Ok(vulnerabilities)
    }

    async fn scan_license_compliance(&self, package_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError> {
        let mut vulnerabilities = Vec::new();

        // Check package.json license
        let package_json_path = format!("{}/package.json", package_path);
        if Path::new(&package_json_path).exists() {
            let package_content = std::fs::read_to_string(&package_json_path)
                .map_err(|e| ModuleServiceError::IoError(e))?;
            
            let package_json: serde_json::Value = serde_json::from_str(&package_content)
                .map_err(|e| ModuleServiceError::SerializationError(e))?;

            if let Some(license) = package_json.get("license").and_then(|l| l.as_str()) {
                if !self.is_license_allowed(license) {
                    vulnerabilities.push(SecurityVulnerability {
                        severity: VulnerabilitySeverity::Low,
                        category: "license".to_string(),
                        description: format!("License '{}' may not be compatible", license),
                        file_path: Some(package_json_path.clone()),
                        line_number: None,
                        recommendation: "Review license compatibility".to_string(),
                    });
                }
            } else {
                vulnerabilities.push(SecurityVulnerability {
                    severity: VulnerabilitySeverity::Low,
                    category: "license".to_string(),
                    description: "No license specified".to_string(),
                    file_path: Some(package_json_path),
                    line_number: None,
                    recommendation: "Specify a license".to_string(),
                });
            }
        }

        // Check for LICENSE files
        let license_files = self.find_files_by_name(package_path, &["LICENSE", "LICENSE.txt", "LICENSE.md"]).await?;
        if license_files.is_empty() {
            vulnerabilities.push(SecurityVulnerability {
                severity: VulnerabilitySeverity::Low,
                category: "license".to_string(),
                description: "No LICENSE file found".to_string(),
                file_path: None,
                line_number: None,
                recommendation: "Include a LICENSE file".to_string(),
            });
        }

        Ok(vulnerabilities)
    }

    async fn validate_signatures(&self, package_path: &str) -> Result<bool, ModuleServiceError> {
        // In a real implementation, this would validate digital signatures
        // For now, we'll simulate signature validation
        
        // Check if package has a signature file
        let sig_path = format!("{}.sig", package_path);
        if !Path::new(&sig_path).exists() {
            return Ok(false);
        }

        // Simulate signature validation
        // In reality, this would use cryptographic verification
        Ok(true)
    }
}

impl SecurityScanner {
    async fn extract_package_for_scan(&self, package_path: &str) -> Result<String, ModuleServiceError> {
        let temp_dir = format!("/tmp/module_scan_{}", uuid::Uuid::new_v4());
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| ModuleServiceError::IoError(e))?;

        // Extract package (assuming it's a tar.gz or zip)
        if package_path.ends_with(".tar.gz") || package_path.ends_with(".tgz") {
            let output = Command::new("tar")
                .args(&["-xzf", package_path, "-C", &temp_dir])
                .output()
                .map_err(|e| ModuleServiceError::IoError(e))?;

            if !output.status.success() {
                return Err(ModuleServiceError::SecurityScanError(
                    format!("Failed to extract tar.gz: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        } else if package_path.ends_with(".zip") {
            let output = Command::new("unzip")
                .args(&["-q", package_path, "-d", &temp_dir])
                .output()
                .map_err(|e| ModuleServiceError::IoError(e))?;

            if !output.status.success() {
                return Err(ModuleServiceError::SecurityScanError(
                    format!("Failed to extract zip: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        } else {
            return Err(ModuleServiceError::SecurityScanError(
                "Unsupported package format".to_string()
            ));
        }

        Ok(temp_dir)
    }

    async fn find_files_by_extension(&self, dir: &str, extensions: &[&str]) -> Result<Vec<String>, ModuleServiceError> {
        let mut files = Vec::new();
        self.find_files_recursive(dir, &mut files, |path| {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                extensions.contains(&ext)
            } else {
                false
            }
        })?;
        Ok(files)
    }

    async fn find_files_by_name(&self, dir: &str, names: &[&str]) -> Result<Vec<String>, ModuleServiceError> {
        let mut files = Vec::new();
        self.find_files_recursive(dir, &mut files, |path| {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                names.contains(&name)
            } else {
                false
            }
        })?;
        Ok(files)
    }

    async fn find_all_text_files(&self, dir: &str) -> Result<Vec<String>, ModuleServiceError> {
        let text_extensions = vec![
            "js", "ts", "jsx", "tsx", "json", "md", "txt", "yml", "yaml", 
            "toml", "ini", "cfg", "conf", "xml", "html", "css", "scss", "less"
        ];
        self.find_files_by_extension(dir, &text_extensions).await
    }

    async fn find_large_files(&self, dir: &str, size_limit: u64) -> Result<Vec<String>, ModuleServiceError> {
        let mut large_files = Vec::new();
        self.find_files_recursive(dir, &mut large_files, |path| {
            if let Ok(metadata) = std::fs::metadata(path) {
                metadata.len() > size_limit
            } else {
                false
            }
        })?;
        Ok(large_files)
    }

    fn find_files_recursive<F>(&self, dir: &str, files: &mut Vec<String>, predicate: F) -> Result<(), ModuleServiceError>
    where
        F: Fn(&Path) -> bool + Copy,
    {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| ModuleServiceError::IoError(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| ModuleServiceError::IoError(e))?;
            let path = entry.path();

            if path.is_dir() {
                self.find_files_recursive(path.to_str().unwrap(), files, predicate)?;
            } else if predicate(&path) {
                files.push(path.to_string_lossy().to_string());
            }
        }

        Ok(())
    }

    async fn scan_javascript_file(&self, file_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError> {
        let mut vulnerabilities = Vec::new();
        
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| ModuleServiceError::IoError(e))?;

        // Check for dangerous functions
        let dangerous_patterns = vec![
            (r"eval\s*\(", VulnerabilitySeverity::High, "Use of eval() function"),
            (r"innerHTML\s*=", VulnerabilitySeverity::Medium, "Direct innerHTML assignment"),
            (r"document\.write\s*\(", VulnerabilitySeverity::Medium, "Use of document.write()"),
            (r"setTimeout\s*\(\s*['\"]", VulnerabilitySeverity::Medium, "String-based setTimeout"),
            (r"setInterval\s*\(\s*['\"]", VulnerabilitySeverity::Medium, "String-based setInterval"),
        ];

        for (pattern, severity, description) in dangerous_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for mat in regex.find_iter(&content) {
                    let line_number = content[..mat.start()].matches('\n').count() + 1;
                    
                    vulnerabilities.push(SecurityVulnerability {
                        severity,
                        category: "dangerous_function".to_string(),
                        description: description.to_string(),
                        file_path: Some(file_path.to_string()),
                        line_number: Some(line_number as u32),
                        recommendation: "Avoid using dangerous functions".to_string(),
                    });
                }
            }
        }

        Ok(vulnerabilities)
    }

    async fn scan_config_file(&self, file_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError> {
        let mut vulnerabilities = Vec::new();
        
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| ModuleServiceError::IoError(e))?;

        // Check for hardcoded secrets
        let secret_patterns = vec![
            (r"password\s*[:=]\s*['\"][^'\"]+['\"]", "Hardcoded password"),
            (r"api[_-]?key\s*[:=]\s*['\"][^'\"]+['\"]", "Hardcoded API key"),
            (r"secret\s*[:=]\s*['\"][^'\"]+['\"]", "Hardcoded secret"),
            (r"token\s*[:=]\s*['\"][^'\"]+['\"]", "Hardcoded token"),
        ];

        for (pattern, description) in secret_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for mat in regex.find_iter(&content) {
                    let line_number = content[..mat.start()].matches('\n').count() + 1;
                    
                    vulnerabilities.push(SecurityVulnerability {
                        severity: VulnerabilitySeverity::High,
                        category: "hardcoded_secret".to_string(),
                        description: description.to_string(),
                        file_path: Some(file_path.to_string()),
                        line_number: Some(line_number as u32),
                        recommendation: "Use environment variables for secrets".to_string(),
                    });
                }
            }
        }

        Ok(vulnerabilities)
    }

    async fn scan_sensitive_data(&self, package_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError> {
        let mut vulnerabilities = Vec::new();
        
        let files = self.find_all_text_files(package_path).await?;
        
        for file_path in files {
            let content = match std::fs::read_to_string(&file_path) {
                Ok(content) => content,
                Err(_) => continue,
            };

            // Check for PII patterns
            let pii_patterns = vec![
                (r"\b\d{3}-\d{2}-\d{4}\b", "Social Security Number"),
                (r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b", "Credit Card Number"),
                (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b", "Email Address"),
                (r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "IP Address"),
            ];

            for (pattern, description) in pii_patterns {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    for mat in regex.find_iter(&content) {
                        let line_number = content[..mat.start()].matches('\n').count() + 1;
                        
                        vulnerabilities.push(SecurityVulnerability {
                            severity: VulnerabilitySeverity::Medium,
                            category: "sensitive_data".to_string(),
                            description: format!("Potential {} found", description),
                            file_path: Some(file_path.clone()),
                            line_number: Some(line_number as u32),
                            recommendation: "Remove or obfuscate sensitive data".to_string(),
                        });
                    }
                }
            }
        }

        Ok(vulnerabilities)
    }

    async fn check_dependency_vulnerabilities(&self, dep_name: &str, dep_version: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError> {
        let mut vulnerabilities = Vec::new();

        // Check against known vulnerable packages
        let vulnerable_packages = self.get_vulnerable_packages();
        
        if let Some(vuln_versions) = vulnerable_packages.get(dep_name) {
            for vuln_version in vuln_versions {
                if self.version_matches(dep_version, &vuln_version.version_pattern) {
                    vulnerabilities.push(SecurityVulnerability {
                        severity: vuln_version.severity.clone(),
                        category: "vulnerable_dependency".to_string(),
                        description: format!("Vulnerable dependency: {} {}", dep_name, dep_version),
                        file_path: Some("package.json".to_string()),
                        line_number: None,
                        recommendation: format!("Update to version {}", vuln_version.fixed_version),
                    });
                }
            }
        }

        Ok(vulnerabilities)
    }

    async fn scan_vulnerable_patterns(&self, package_path: &str) -> Result<Vec<SecurityVulnerability>, ModuleServiceError> {
        let mut vulnerabilities = Vec::new();
        
        // This would scan for known vulnerable code patterns
        // For now, we'll return an empty list
        
        Ok(vulnerabilities)
    }

    fn calculate_security_score(&self, vulnerabilities: &[SecurityVulnerability]) -> u8 {
        let mut score = 100u8;

        for vuln in vulnerabilities {
            let deduction = match vuln.severity {
                VulnerabilitySeverity::Critical => 25,
                VulnerabilitySeverity::High => 15,
                VulnerabilitySeverity::Medium => 8,
                VulnerabilitySeverity::Low => 3,
            };
            
            score = score.saturating_sub(deduction);
        }

        score
    }

    fn get_pattern_severity(&self, pattern: &str) -> VulnerabilitySeverity {
        match pattern {
            p if p.contains("eval") => VulnerabilitySeverity::High,
            p if p.contains("child_process") => VulnerabilitySeverity::High,
            p if p.contains("fs.") => VulnerabilitySeverity::Medium,
            _ => VulnerabilitySeverity::Low,
        }
    }

    fn is_license_allowed(&self, license: &str) -> bool {
        let allowed_licenses = vec![
            "MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", 
            "ISC", "Unlicense", "CC0-1.0"
        ];
        
        allowed_licenses.contains(&license)
    }

    fn get_vulnerable_packages(&self) -> HashMap<String, Vec<VulnerableVersion>> {
        // This would be loaded from a vulnerability database
        // For now, we'll return a small sample
        let mut vulnerable = HashMap::new();
        
        vulnerable.insert("lodash".to_string(), vec![
            VulnerableVersion {
                version_pattern: "<4.17.12".to_string(),
                severity: VulnerabilitySeverity::High,
                fixed_version: "4.17.12".to_string(),
            }
        ]);

        vulnerable
    }

    fn version_matches(&self, version: &str, pattern: &str) -> bool {
        // Simple version matching - in production would use proper semver
        if pattern.starts_with('<') {
            let target_version = &pattern[1..];
            version < target_version
        } else if pattern.starts_with('>') {
            let target_version = &pattern[1..];
            version > target_version
        } else {
            version == pattern
        }
    }
}

// Supporting types
#[derive(Debug, Clone)]
pub struct ScannerRules {
    pub max_file_size: u64,
    pub allowed_extensions: Vec<String>,
    pub blocked_patterns: Vec<String>,
}

impl Default for ScannerRules {
    fn default() -> Self {
        Self {
            max_file_size: 50 * 1024 * 1024, // 50MB
            allowed_extensions: vec![
                "js".to_string(), "ts".to_string(), "jsx".to_string(), "tsx".to_string(),
                "json".to_string(), "md".to_string(), "txt".to_string(),
                "css".to_string(), "scss".to_string(), "less".to_string(),
                "html".to_string(), "xml".to_string(), "yml".to_string(), "yaml".to_string(),
            ],
            blocked_patterns: vec![
                r"\.exe$".to_string(),
                r"\.dll$".to_string(),
                r"\.so$".to_string(),
                r"\.dylib$".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct VulnerableVersion {
    pub version_pattern: String,
    pub severity: VulnerabilitySeverity,
    pub fixed_version: String,
}

// Entry point for the scanner service
pub async fn start_scanner(config: ModuleServiceConfig) -> Result<(), ModuleServiceError> {
    info!("Starting module security scanner service");

    let scanner = SecurityScanner::new(config);

    // In a real implementation, this would start a service that:
    // 1. Listens for scan requests
    // 2. Processes scan queue
    // 3. Updates vulnerability database
    // 4. Provides scan results API

    info!("Security scanner service started");
    
    // Keep the service running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}