use std::sync::OnceLock;

use indicatif::ProgressStyle;
use ureq::{Agent, AgentBuilder};

pub const EMAIL: &str = "automated@joel.net.au";
pub const USER_AGENT: &str =
    "priceshark-database/0.1.0 (+https://github.com/priceshark/database +mailto:automated@joel.net.au)";

pub fn agent() -> Agent {
    static AGENT: OnceLock<Agent> = OnceLock::new();
    AGENT
        .get_or_init(|| AgentBuilder::new().user_agent(USER_AGENT).build())
        .clone()
}

pub fn progress_style() -> ProgressStyle {
    ProgressStyle::with_template("{percent}% {human_pos}/{human_len} {per_sec} ({eta_precise})")
        .expect("hardcoded")
}
