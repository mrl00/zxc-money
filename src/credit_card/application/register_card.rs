use std::sync::Arc;

use crate::credit_card::domain::card::CreditCard;
use crate::credit_card::domain::repository::CreditCardRepository;
use crate::provider::id::IdGenerator;
use crate::shared::errors::CreditCardError;
use crate::shared::ids::{CreditCardID, Principal};
use crate::shared::money::Money;

/// Command to register a new credit card.
pub struct RegisterCardCommand {
    pub principal: Principal,
    pub name: String,
    pub brand: String,
    pub limit: Money,
    pub closing_day: u32,
    pub due_day: u32,
}

/// Handler that processes [`RegisterCardCommand`] requests.
pub struct RegisterCardHandler<C: CreditCardRepository, I: IdGenerator> {
    credit_card_repository: Arc<C>,
    id_generator: Arc<I>,
}

impl<C: CreditCardRepository, I: IdGenerator> RegisterCardHandler<C, I> {
    pub fn new(credit_card_repository: Arc<C>, id_generator: Arc<I>) -> Self {
        Self {
            credit_card_repository,
            id_generator,
        }
    }

    /// Creates a new credit card and persists it.
    pub async fn handle(&self, cmd: RegisterCardCommand) -> Result<CreditCardID, CreditCardError> {
        let id = CreditCardID::from_uuid(self.id_generator.new_id());

        let card = CreditCard::new(
            id,
            cmd.principal.user_id,
            cmd.name,
            cmd.brand,
            cmd.limit,
            cmd.closing_day,
            cmd.due_day,
        );

        self.credit_card_repository.save(&card).await?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::UserID;
    use crate::shared::mock::MockCreditCardRepository;
    use crate::shared::money::Currency;

    #[tokio::test]
    async fn test_register_card() {
        let repo = Arc::new(MockCreditCardRepository::new());
        let id_gen = Arc::new(crate::provider::id::MockIdGenerator::new(
            uuid::Uuid::new_v4(),
        ));
        let handler = RegisterCardHandler::new(repo.clone(), id_gen);

        let id = handler
            .handle(RegisterCardCommand {
                principal: Principal::new(UserID::new()),
                name: "Nubank".into(),
                brand: "Mastercard".into(),
                limit: Money::from_cents(500000, Currency::BRL),
                closing_day: 20,
                due_day: 27,
            })
            .await
            .unwrap();

        let card = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(card.name, "Nubank");
    }
}
