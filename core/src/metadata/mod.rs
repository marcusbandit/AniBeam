//! Provider records: the pure translation from a provider's reply to the
//! rows behind it. Every match, every refresh and every crawl step writes
//! through here, so the rules live in one place rather than once per job.

pub mod record;
