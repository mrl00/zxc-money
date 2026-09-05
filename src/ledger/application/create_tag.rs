use std::sync::Arc;

use crate::ledger::domain::repository::TagRepository;
use crate::shared::errors::LedgerError;
use crate::shared::ids::{Principal, TagID};

/// Command to create a new tag (or find an existing one by name).
pub struct CreateTagCommand {
    pub principal: Principal,
    pub name: String,
}

/// Handler that processes [`CreateTagCommand`] requests.
pub struct CreateTagHandler<T: TagRepository> {
    tag_repository: Arc<T>,
}

impl<T: TagRepository> CreateTagHandler<T> {
    pub fn new(tag_repository: Arc<T>) -> Self {
        Self { tag_repository }
    }

    /// Finds or creates a tag by name and returns its ID.
    pub async fn handle(&self, cmd: CreateTagCommand) -> Result<TagID, LedgerError> {
        let tag = self.tag_repository.find_or_create(cmd.name).await?;
        Ok(tag.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_tag_handler_compiles() {
        let tag_id = TagID::new();
        assert!(!tag_id.as_uuid().is_nil());
    }
}
