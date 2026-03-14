//! Alert system for notifications and monitoring
//!
//! Provides trait-based alerting with multiple backend implementations.

use sig_types::{Result, SigcError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Error => write!(f, "ERROR"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// An alert message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Alert {
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub source: String,
    pub timestamp: u64,
    pub tags: HashMap<String, String>,
}

impl Alert {
    /// Create a new alert
    pub fn new(severity: AlertSeverity, title: &str, message: &str) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        Alert {
            severity,
            title: title.to_string(),
            message: message.to_string(),
            source: "sigc".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tags: HashMap::new(),
        }
    }

    pub fn info(title: &str, message: &str) -> Self {
        Self::new(AlertSeverity::Info, title, message)
    }

    pub fn warning(title: &str, message: &str) -> Self {
        Self::new(AlertSeverity::Warning, title, message)
    }

    pub fn error(title: &str, message: &str) -> Self {
        Self::new(AlertSeverity::Error, title, message)
    }

    pub fn critical(title: &str, message: &str) -> Self {
        Self::new(AlertSeverity::Critical, title, message)
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }

    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }
}

/// Trait for alert sinks
pub trait AlertSink: Send + Sync {
    /// Send an alert
    fn send(&self, alert: &Alert) -> Result<()>;

    /// Get sink name
    fn name(&self) -> &str;

    /// Check if sink is available
    fn is_available(&self) -> bool;
}

/// Console alert sink (prints to stderr)
pub struct ConsoleAlertSink {
    name: String,
}

impl ConsoleAlertSink {
    pub fn new() -> Self {
        ConsoleAlertSink {
            name: "console".to_string(),
        }
    }
}

impl Default for ConsoleAlertSink {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertSink for ConsoleAlertSink {
    fn send(&self, alert: &Alert) -> Result<()> {
        eprintln!(
            "[{}] {} - {}: {}",
            alert.severity, alert.source, alert.title, alert.message
        );
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// Slack alert sink
pub struct SlackAlertSink {
    webhook_url: String,
    channel: Option<String>,
    username: Option<String>,
    name: String,
}

impl SlackAlertSink {
    /// Create a new Slack alert sink
    pub fn new(webhook_url: &str) -> Self {
        SlackAlertSink {
            webhook_url: webhook_url.to_string(),
            channel: None,
            username: None,
            name: "slack".to_string(),
        }
    }

    /// Create from environment variable SLACK_WEBHOOK_URL
    pub fn from_env() -> Option<Self> {
        std::env::var("SLACK_WEBHOOK_URL")
            .ok()
            .map(|url| Self::new(&url))
    }

    /// Set the channel to post to
    pub fn with_channel(mut self, channel: &str) -> Self {
        self.channel = Some(channel.to_string());
        self
    }

    /// Set the username to post as
    pub fn with_username(mut self, username: &str) -> Self {
        self.username = Some(username.to_string());
        self
    }

    fn severity_emoji(&self, severity: AlertSeverity) -> &str {
        match severity {
            AlertSeverity::Info => ":information_source:",
            AlertSeverity::Warning => ":warning:",
            AlertSeverity::Error => ":x:",
            AlertSeverity::Critical => ":rotating_light:",
        }
    }

    fn severity_color(&self, severity: AlertSeverity) -> &str {
        match severity {
            AlertSeverity::Info => "#36a64f",
            AlertSeverity::Warning => "#daa038",
            AlertSeverity::Error => "#cc0000",
            AlertSeverity::Critical => "#ff0000",
        }
    }
}

impl AlertSink for SlackAlertSink {
    fn send(&self, alert: &Alert) -> Result<()> {
        let emoji = self.severity_emoji(alert.severity);
        let color = self.severity_color(alert.severity);

        let mut payload = serde_json::json!({
            "attachments": [{
                "color": color,
                "title": format!("{} {}", emoji, alert.title),
                "text": alert.message,
                "fields": [
                    {
                        "title": "Severity",
                        "value": alert.severity.to_string(),
                        "short": true
                    },
                    {
                        "title": "Source",
                        "value": alert.source,
                        "short": true
                    }
                ],
                "footer": "sigc alerting",
                "ts": alert.timestamp
            }]
        });

        // Add channel if specified
        if let Some(ref channel) = self.channel {
            payload["channel"] = serde_json::Value::String(channel.clone());
        }

        // Add username if specified
        if let Some(ref username) = self.username {
            payload["username"] = serde_json::Value::String(username.clone());
        }

        // Send via blocking HTTP request
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .map_err(|e| SigcError::Runtime(format!("Slack request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SigcError::Runtime(format!(
                "Slack returned error: {}",
                response.status()
            )));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        !self.webhook_url.is_empty()
    }
}

/// Email alert sink
#[allow(dead_code)]
pub struct EmailAlertSink {
    smtp_server: String,
    from: String,
    to: Vec<String>,
    name: String,
}

impl EmailAlertSink {
    pub fn new(smtp_server: &str, from: &str, to: Vec<String>) -> Self {
        EmailAlertSink {
            smtp_server: smtp_server.to_string(),
            from: from.to_string(),
            to,
            name: "email".to_string(),
        }
    }
}

impl AlertSink for EmailAlertSink {
    fn send(&self, _alert: &Alert) -> Result<()> {
        // Email implementation would go here
        // For now, just return Ok as a placeholder
        Err(SigcError::Runtime("Email alerting not yet implemented".into()))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        false // Not implemented
    }
}

/// Mock alert sink for testing
pub struct MockAlertSink {
    sent: Arc<RwLock<Vec<Alert>>>,
    name: String,
}

impl MockAlertSink {
    pub fn new() -> Self {
        MockAlertSink {
            sent: Arc::new(RwLock::new(Vec::new())),
            name: "mock".to_string(),
        }
    }

    /// Get all sent alerts
    pub fn get_alerts(&self) -> Vec<Alert> {
        self.sent.read().unwrap().clone()
    }

    /// Clear sent alerts
    pub fn clear(&self) {
        self.sent.write().unwrap().clear();
    }

    /// Get count of alerts sent
    pub fn count(&self) -> usize {
        self.sent.read().unwrap().len()
    }
}

impl Default for MockAlertSink {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertSink for MockAlertSink {
    fn send(&self, alert: &Alert) -> Result<()> {
        self.sent.write().unwrap().push(alert.clone());
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// Alert manager for routing alerts to multiple sinks
pub struct AlertManager {
    sinks: HashMap<String, Box<dyn AlertSink>>,
    rules: Vec<AlertRule>,
}

/// Rule for routing alerts
pub struct AlertRule {
    /// Minimum severity to match
    pub min_severity: AlertSeverity,
    /// Sinks to route to
    pub sinks: Vec<String>,
    /// Optional tag filter (key=value)
    pub tag_filter: Option<(String, String)>,
}

impl AlertManager {
    pub fn new() -> Self {
        AlertManager {
            sinks: HashMap::new(),
            rules: Vec::new(),
        }
    }

    /// Register an alert sink
    pub fn register(&mut self, name: &str, sink: Box<dyn AlertSink>) {
        self.sinks.insert(name.to_string(), sink);
    }

    /// Add a routing rule
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    /// Send an alert, routing to appropriate sinks
    pub fn send(&self, alert: &Alert) -> Result<()> {
        let mut sent_to: Vec<String> = Vec::new();

        for rule in &self.rules {
            // Check severity
            if !matches_severity(alert.severity, rule.min_severity) {
                continue;
            }

            // Check tag filter
            if let Some((ref key, ref value)) = rule.tag_filter {
                if alert.tags.get(key) != Some(value) {
                    continue;
                }
            }

            // Send to matched sinks
            for sink_name in &rule.sinks {
                if sent_to.contains(sink_name) {
                    continue; // Already sent
                }

                if let Some(sink) = self.sinks.get(sink_name) {
                    if sink.is_available() {
                        sink.send(alert)?;
                        sent_to.push(sink_name.clone());
                    }
                }
            }
        }

        // If no rules matched, send to all available sinks
        if sent_to.is_empty() {
            for (name, sink) in &self.sinks {
                if sink.is_available() && !sent_to.contains(name) {
                    sink.send(alert)?;
                }
            }
        }

        Ok(())
    }

    /// Send directly to a specific sink
    pub fn send_to(&self, sink_name: &str, alert: &Alert) -> Result<()> {
        let sink = self.sinks.get(sink_name)
            .ok_or_else(|| SigcError::Runtime(format!("Sink not found: {}", sink_name)))?;
        sink.send(alert)
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

fn matches_severity(alert_severity: AlertSeverity, min_severity: AlertSeverity) -> bool {
    let alert_level = severity_level(alert_severity);
    let min_level = severity_level(min_severity);
    alert_level >= min_level
}

fn severity_level(severity: AlertSeverity) -> u8 {
    match severity {
        AlertSeverity::Info => 0,
        AlertSeverity::Warning => 1,
        AlertSeverity::Error => 2,
        AlertSeverity::Critical => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert::info("Test", "This is a test")
            .with_source("test_runner")
            .with_tag("env", "test");

        assert_eq!(alert.severity, AlertSeverity::Info);
        assert_eq!(alert.title, "Test");
        assert_eq!(alert.source, "test_runner");
        assert_eq!(alert.tags.get("env"), Some(&"test".to_string()));
    }

    #[test]
    fn test_console_sink() {
        let sink = ConsoleAlertSink::new();
        let alert = Alert::info("Test", "Message");

        assert!(sink.is_available());
        assert!(sink.send(&alert).is_ok());
    }

    #[test]
    fn test_mock_sink() {
        let sink = MockAlertSink::new();

        sink.send(&Alert::info("Test 1", "First")).unwrap();
        sink.send(&Alert::warning("Test 2", "Second")).unwrap();

        assert_eq!(sink.count(), 2);

        let alerts = sink.get_alerts();
        assert_eq!(alerts[0].title, "Test 1");
        assert_eq!(alerts[1].severity, AlertSeverity::Warning);

        sink.clear();
        assert_eq!(sink.count(), 0);
    }

    #[test]
    fn test_alert_manager() {
        let mut manager = AlertManager::new();
        let mock = MockAlertSink::new();
        let mock_alerts = mock.sent.clone();

        manager.register("mock", Box::new(mock));

        let alert = Alert::error("Failure", "Something failed");
        manager.send(&alert).unwrap();

        assert_eq!(mock_alerts.read().unwrap().len(), 1);
    }

    #[test]
    fn test_alert_routing() {
        let mut manager = AlertManager::new();

        let critical_mock = MockAlertSink::new();
        let critical_alerts = critical_mock.sent.clone();

        let all_mock = MockAlertSink::new();
        let all_alerts = all_mock.sent.clone();

        manager.register("critical_sink", Box::new(critical_mock));
        manager.register("all_sink", Box::new(all_mock));

        // Route critical alerts only to critical_sink
        manager.add_rule(AlertRule {
            min_severity: AlertSeverity::Critical,
            sinks: vec!["critical_sink".to_string()],
            tag_filter: None,
        });

        // Route all alerts to all_sink
        manager.add_rule(AlertRule {
            min_severity: AlertSeverity::Info,
            sinks: vec!["all_sink".to_string()],
            tag_filter: None,
        });

        // Send info alert
        manager.send(&Alert::info("Info", "Info message")).unwrap();
        assert_eq!(critical_alerts.read().unwrap().len(), 0);
        assert_eq!(all_alerts.read().unwrap().len(), 1);

        // Send critical alert
        manager.send(&Alert::critical("Critical", "Critical message")).unwrap();
        assert_eq!(critical_alerts.read().unwrap().len(), 1);
        assert_eq!(all_alerts.read().unwrap().len(), 2);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(AlertSeverity::Info.to_string(), "INFO");
        assert_eq!(AlertSeverity::Warning.to_string(), "WARNING");
        assert_eq!(AlertSeverity::Error.to_string(), "ERROR");
        assert_eq!(AlertSeverity::Critical.to_string(), "CRITICAL");
    }

    #[test]
    fn test_severity_matching() {
        assert!(matches_severity(AlertSeverity::Critical, AlertSeverity::Info));
        assert!(matches_severity(AlertSeverity::Error, AlertSeverity::Warning));
        assert!(!matches_severity(AlertSeverity::Info, AlertSeverity::Error));
    }
}
