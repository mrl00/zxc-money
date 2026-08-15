use uuid::Uuid;

pub trait IdGenerator: Send + Sync {
    fn new_id(&self) -> Uuid;
}

pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    fn new_id(&self) -> Uuid {
        Uuid::new_v4()
    }
}

#[cfg(test)]
pub struct MockIdGenerator {
    pub next_id: Uuid,
}

#[cfg(test)]
impl MockIdGenerator {
    pub fn new(next_id: Uuid) -> Self {
        Self { next_id }
    }
}

#[cfg(test)]
impl IdGenerator for MockIdGenerator {
    fn new_id(&self) -> Uuid {
        self.next_id
    }
}
