use std::time::Instant;

#[derive(Clone, Debug)]
pub struct HealthStatus {
    pub healthy: bool,
    pub response_time_ms: u64,
    pub last_check: Instant,
    pub error_message: Option<String>,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            healthy: false,
            response_time_ms: 0,
            last_check: Instant::now(),
            error_message: None,
        }
    }
}
