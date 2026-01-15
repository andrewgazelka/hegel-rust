//! Format-specific string generators for emails, URLs, IPs, dates, etc.

use super::{generate_from_schema, Generate};
use serde_json::{json, Value};

// ============================================================================
// Email Generator
// ============================================================================

/// Generator for email addresses.
pub struct EmailGenerator;

impl Generate<String> for EmailGenerator {
    fn generate(&self) -> String {
        generate_from_schema(&self.schema().unwrap())
    }

    fn schema(&self) -> Option<Value> {
        Some(json!({"type": "string", "format": "email"}))
    }
}

/// Generate email addresses.
pub fn emails() -> EmailGenerator {
    EmailGenerator
}

// ============================================================================
// URL Generator
// ============================================================================

/// Generator for URLs.
pub struct UrlGenerator;

impl Generate<String> for UrlGenerator {
    fn generate(&self) -> String {
        generate_from_schema(&self.schema().unwrap())
    }

    fn schema(&self) -> Option<Value> {
        Some(json!({"type": "string", "format": "uri"}))
    }
}

/// Generate URLs.
pub fn urls() -> UrlGenerator {
    UrlGenerator
}

// ============================================================================
// Domain Generator
// ============================================================================

/// Generator for domain names.
pub struct DomainGenerator {
    max_length: usize,
}

impl DomainGenerator {
    /// Set the maximum length.
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }
}

impl Generate<String> for DomainGenerator {
    fn generate(&self) -> String {
        generate_from_schema(&self.schema().unwrap())
    }

    fn schema(&self) -> Option<Value> {
        Some(json!({
            "type": "string",
            "format": "hostname",
            "maxLength": self.max_length
        }))
    }
}

/// Generate domain names.
pub fn domains() -> DomainGenerator {
    DomainGenerator { max_length: 255 }
}

// ============================================================================
// IP Address Generator
// ============================================================================

/// IP address version.
#[derive(Clone, Copy)]
pub enum IpVersion {
    V4,
    V6,
}

/// Generator for IP addresses.
pub struct IpAddressGenerator {
    version: Option<IpVersion>,
}

impl IpAddressGenerator {
    /// Generate only IPv4 addresses.
    pub fn v4(mut self) -> Self {
        self.version = Some(IpVersion::V4);
        self
    }

    /// Generate only IPv6 addresses.
    pub fn v6(mut self) -> Self {
        self.version = Some(IpVersion::V6);
        self
    }
}

impl Generate<String> for IpAddressGenerator {
    fn generate(&self) -> String {
        generate_from_schema(&self.schema().unwrap())
    }

    fn schema(&self) -> Option<Value> {
        match self.version {
            Some(IpVersion::V4) => Some(json!({"type": "string", "format": "ipv4"})),
            Some(IpVersion::V6) => Some(json!({"type": "string", "format": "ipv6"})),
            None => Some(json!({
                "anyOf": [
                    {"type": "string", "format": "ipv4"},
                    {"type": "string", "format": "ipv6"}
                ]
            })),
        }
    }
}

/// Generate IP addresses.
///
/// By default generates either IPv4 or IPv6. Use `.v4()` or `.v6()` to constrain.
pub fn ip_addresses() -> IpAddressGenerator {
    IpAddressGenerator { version: None }
}

// ============================================================================
// Date Generator
// ============================================================================

/// Generator for ISO 8601 dates (YYYY-MM-DD).
pub struct DateGenerator;

impl Generate<String> for DateGenerator {
    fn generate(&self) -> String {
        generate_from_schema(&self.schema().unwrap())
    }

    fn schema(&self) -> Option<Value> {
        Some(json!({"type": "string", "format": "date"}))
    }
}

/// Generate ISO 8601 dates.
pub fn dates() -> DateGenerator {
    DateGenerator
}

// ============================================================================
// Time Generator
// ============================================================================

/// Generator for ISO 8601 times (HH:MM:SS).
pub struct TimeGenerator;

impl Generate<String> for TimeGenerator {
    fn generate(&self) -> String {
        generate_from_schema(&self.schema().unwrap())
    }

    fn schema(&self) -> Option<Value> {
        Some(json!({"type": "string", "format": "time"}))
    }
}

/// Generate ISO 8601 times.
pub fn times() -> TimeGenerator {
    TimeGenerator
}

// ============================================================================
// DateTime Generator
// ============================================================================

/// Generator for ISO 8601 datetimes.
pub struct DateTimeGenerator;

impl Generate<String> for DateTimeGenerator {
    fn generate(&self) -> String {
        generate_from_schema(&self.schema().unwrap())
    }

    fn schema(&self) -> Option<Value> {
        Some(json!({"type": "string", "format": "date-time"}))
    }
}

/// Generate ISO 8601 datetimes.
pub fn datetimes() -> DateTimeGenerator {
    DateTimeGenerator
}
