use serde::Serialize;

pub const BUILD_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize)]
pub struct BugReportPayload<'a> {
    pub game: &'static str,
    pub version: &'static str,
    pub phase: &'a str,
    pub platform: &'static str,
}

pub fn bug_report_payload(phase: &str) -> String {
    let payload = BugReportPayload {
        game: "nanite_swarm",
        version: BUILD_VERSION,
        phase,
        platform: std::env::consts::OS,
    };
    serde_json::to_string(&payload).expect("bug report payload is serializable")
}
