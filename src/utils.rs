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

pub fn title_case(s: &str) -> String {
    let mut should_caps: bool = true;
    let mut new = String::new();
    for x in s.chars() {
        let x = if should_caps {
            should_caps = false;
            x
        } else {
            if x == ' ' {
                should_caps = true;
            }
            x.to_ascii_lowercase()
        };
        new.push(x);
    }
    new
}

pub fn slug(s: &str) -> String {
    s.chars()
        .into_iter()
        .map(|x| match x {
            ' ' => '-',
            'A'..='Z' => x.to_ascii_lowercase(),
            x => x,
        })
        .collect()
}
