//! The one network call in the app: a person-initiated read of a calendar
//! subscription they added. Errors are classified and never include the link.
use temporal_core::domain::AttemptOutcome;

pub const MAX_FEED_BYTES: u64 = crate::ics::MAX_BYTES as u64;

pub fn fetch_feed(url: &str) -> Result<String, (AttemptOutcome, String)> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(30)))
        .user_agent("TemporalEngine/0.1 (personal calendar; read-only)")
        .max_redirects(5)
        .build()
        .into();
    let mut response = agent.get(url).call().map_err(|error| match error {
        ureq::Error::StatusCode(401 | 403) => (
            AttemptOutcome::Failed,
            "The calendar refused access. The link may have been reset; paste a new one.".into(),
        ),
        ureq::Error::StatusCode(404 | 410) => (
            AttemptOutcome::Failed,
            "The calendar link no longer exists.".into(),
        ),
        ureq::Error::StatusCode(429) => (
            AttemptOutcome::Failed,
            "The calendar asked to slow down. Last-known events are kept; try later.".into(),
        ),
        ureq::Error::StatusCode(code) => (
            AttemptOutcome::Failed,
            format!("The calendar server answered with an error ({code})."),
        ),
        ureq::Error::Timeout(_) => (
            AttemptOutcome::Failed,
            "The calendar took too long to answer. Last-known events are kept.".into(),
        ),
        _ => (
            AttemptOutcome::Failed,
            "Could not reach the calendar. You may be offline; last-known events are kept.".into(),
        ),
    })?;
    response
        .body_mut()
        .with_config()
        .limit(MAX_FEED_BYTES)
        .read_to_string()
        .map_err(|_| {
            (
                AttemptOutcome::Partial,
                "The calendar download was incomplete or too large; last-known events are kept."
                    .into(),
            )
        })
}
