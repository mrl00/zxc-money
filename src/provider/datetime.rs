use chrono::{DateTime, Utc};

pub trait DateTimeProvider: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemDateTime;

impl DateTimeProvider for SystemDateTime {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
pub struct MockDateTime {
    pub fixed_now: DateTime<Utc>,
}

#[cfg(test)]
impl MockDateTime {
    pub fn new(fixed_now: DateTime<Utc>) -> Self {
        Self { fixed_now }
    }
}

#[cfg(test)]
impl DateTimeProvider for MockDateTime {
    fn now(&self) -> DateTime<Utc> {
        self.fixed_now
    }
}
