use chrono::Utc;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tokio::time::Instant;
use tracing::info;

/// Simple health check system - replaces complex metrics collection
/// Focused on basic health status for MCP server needs
pub struct SimpleHealthChecker {
    start_instant: Instant,
}

/// Basic health status for MCP operations
#[derive(Debug, Clone)]
pub struct BasicHealthStatus {
    pub is_healthy: bool,
    pub uptime_seconds: u64,
    pub message: String,
}

impl Default for SimpleHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleHealthChecker {
    /// Create a new simple health checker
    pub fn new() -> Self {
        Self {
            start_instant: Instant::now(),
        }
    }

    /// Perform a basic health check - just verify the server is responsive
    pub async fn check_health(&self) -> BasicHealthStatus {
        let uptime_seconds = self.start_instant.elapsed().as_secs();

        // For MCP server, we just need to verify basic responsiveness
        // Real health would check YNAB API connectivity, but that's done at startup
        BasicHealthStatus {
            is_healthy: true, // If we can respond, we're healthy
            uptime_seconds,
            message: format!("MCP server running for {} seconds", uptime_seconds),
        }
    }

    /// Get basic server statistics as JSON
    pub async fn get_health_json(&self) -> Value {
        let health = self.check_health().await;

        json!({
            "status": if health.is_healthy { "healthy" } else { "unhealthy" },
            "uptime_seconds": health.uptime_seconds,
            "uptime_human": format_duration(health.uptime_seconds),
            "message": health.message,
            "timestamp": Utc::now(),
            "server_type": "ynab-mcp"
        })
    }
}

/// Format duration in human-readable form
fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    } else {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

/// Global health checker instance
static HEALTH_CHECKER: OnceLock<SimpleHealthChecker> = OnceLock::new();

/// Get the global health checker instance
pub fn get_health_checker() -> &'static SimpleHealthChecker {
    HEALTH_CHECKER.get_or_init(|| {
        info!("Initializing simple health checker");
        SimpleHealthChecker::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_simple_health_checker_creation() {
        let checker = SimpleHealthChecker::new();
        let health = checker.check_health().await;

        assert!(health.is_healthy);
        assert_eq!(health.uptime_seconds, 0); // Should be 0 or very small
        assert!(health.message.contains("MCP server running"));
    }

    #[tokio::test]
    async fn test_health_checker_default() {
        let checker = SimpleHealthChecker::default();
        let health = checker.check_health().await;

        assert!(health.is_healthy);
        assert!(health.message.contains("MCP server running"));
    }

    #[tokio::test]
    async fn test_health_check_uptime() {
        let checker = SimpleHealthChecker::new();

        // Wait a small amount to test uptime tracking
        sleep(Duration::from_millis(10)).await;

        let health = checker.check_health().await;
        assert!(health.is_healthy);
        // Uptime should be very small but measurable
        assert!(health.uptime_seconds < 10); // Should be less than 10 seconds in tests
    }

    #[tokio::test]
    async fn test_get_health_json() {
        let checker = SimpleHealthChecker::new();
        let health_json = checker.get_health_json().await;

        assert_eq!(health_json["status"], "healthy");
        assert!(health_json["uptime_seconds"].is_number());
        assert!(health_json["uptime_human"].is_string());
        assert!(health_json["message"].is_string());
        assert!(health_json["timestamp"].is_string());
        assert_eq!(health_json["server_type"], "ynab-mcp");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(3660), "1h 1m");
        assert_eq!(format_duration(7200), "2h 0m");
        assert_eq!(format_duration(86399), "23h 59m");
    }

    #[test]
    fn test_format_duration_days() {
        assert_eq!(format_duration(86400), "1d 0h");
        assert_eq!(format_duration(90000), "1d 1h");
        assert_eq!(format_duration(172800), "2d 0h");
        assert_eq!(format_duration(259200), "3d 0h");
    }

    #[test]
    fn test_basic_health_status_clone() {
        let status = BasicHealthStatus {
            is_healthy: true,
            uptime_seconds: 100,
            message: "Test message".to_string(),
        };

        let cloned = status.clone();
        assert_eq!(status.is_healthy, cloned.is_healthy);
        assert_eq!(status.uptime_seconds, cloned.uptime_seconds);
        assert_eq!(status.message, cloned.message);
    }

    #[tokio::test]
    async fn test_get_health_checker_global() {
        let checker1 = get_health_checker();
        let checker2 = get_health_checker();

        // Should be the same instance
        assert!(std::ptr::eq(checker1, checker2));

        let health = checker1.check_health().await;
        assert!(health.is_healthy);
    }

    #[tokio::test]
    async fn test_health_checker_json_fields() {
        let checker = SimpleHealthChecker::new();
        let json = checker.get_health_json().await;

        // Verify all expected fields are present
        let required_fields = [
            "status",
            "uptime_seconds",
            "uptime_human",
            "message",
            "timestamp",
            "server_type",
        ];
        for field in required_fields {
            assert!(json.get(field).is_some(), "Missing field: {}", field);
        }

        // Verify field types
        assert!(json["status"].is_string());
        assert!(json["uptime_seconds"].is_number());
        assert!(json["uptime_human"].is_string());
        assert!(json["message"].is_string());
        assert!(json["timestamp"].is_string());
        assert!(json["server_type"].is_string());
    }
}
