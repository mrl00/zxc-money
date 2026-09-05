use chrono::NaiveDate;

use crate::credit_card::domain::events::PurchaseAdded;
use crate::credit_card::domain::invoice::Invoice;
use crate::credit_card::domain::purchase::Purchase;
use crate::credit_card::domain::repository::{CreditCardRepository, InvoiceRepository};
use crate::provider::id::IdGenerator;
use crate::shared::errors::CreditCardError;
use crate::shared::events::EventPublisher;
use crate::shared::ids::{
    CategoryID, CreditCardID, InstallmentGroupID, InvoiceID, PurchaseID, UserID,
};
use crate::shared::money::Money;
use crate::shared::period::YearMonth;
use std::sync::Arc;

/// Command to register a new purchase on a credit card.
pub struct RegisterPurchaseCommand {
    pub owner_id: UserID,
    pub credit_card_id: CreditCardID,
    pub description: String,
    pub total_amount: Money,
    pub installments_count: u32,
    pub category_id: CategoryID,
    pub purchased_at: NaiveDate,
}

/// Handles [`RegisterPurchaseCommand`] by creating or appending to invoices
/// and publishing [`PurchaseAdded`] events.
///
/// When `installments_count > 1`, the purchase is split across consecutive
/// invoices (one installment per invoice), linked by [`InstallmentGroupID`].
pub struct RegisterPurchaseHandler<
    C: CreditCardRepository,
    I: InvoiceRepository,
    P: EventPublisher,
    Id: IdGenerator,
> {
    credit_card_repository: Arc<C>,
    invoice_repository: Arc<I>,
    event_publisher: Arc<P>,
    id_generator: Arc<Id>,
}

impl<C: CreditCardRepository, I: InvoiceRepository, P: EventPublisher, Id: IdGenerator>
    RegisterPurchaseHandler<C, I, P, Id>
{
    /// Creates a new [`RegisterPurchaseHandler`] with the given dependencies.
    pub fn new(
        credit_card_repository: Arc<C>,
        invoice_repository: Arc<I>,
        event_publisher: Arc<P>,
        id_generator: Arc<Id>,
    ) -> Self {
        Self {
            credit_card_repository,
            invoice_repository,
            event_publisher,
            id_generator,
        }
    }

    /// Executes the register-purchase use case.
    ///
    /// For single-installment purchases, adds to the current open invoice.
    /// For multi-installment purchases, splits across consecutive invoices.
    pub async fn handle(
        &self,
        cmd: RegisterPurchaseCommand,
    ) -> Result<Vec<PurchaseID>, CreditCardError> {
        let card = self
            .credit_card_repository
            .find_by_id(cmd.credit_card_id)
            .await?
            .ok_or_else(|| CreditCardError::CreditCardNotFound(cmd.credit_card_id.to_string()))?;

        if card.owner_id != cmd.owner_id {
            return Err(CreditCardError::InvariantViolation(
                "credit card does not belong to owner".into(),
            ));
        }

        if cmd.installments_count == 0 {
            return Err(CreditCardError::InvariantViolation(
                "installments_count must be >= 1".into(),
            ));
        }

        if cmd.installments_count == 1 {
            self.handle_single_installment(cmd).await
        } else {
            self.handle_multi_installment(cmd).await
        }
    }

    async fn handle_single_installment(
        &self,
        cmd: RegisterPurchaseCommand,
    ) -> Result<Vec<PurchaseID>, CreditCardError> {
        let purchase_id = PurchaseID::from_uuid(self.id_generator.new_id());
        let purchase = Purchase::new(
            purchase_id,
            cmd.description,
            cmd.total_amount,
            1,
            cmd.category_id,
            cmd.purchased_at,
        );

        let reference_month = YearMonth::from_date(cmd.purchased_at);

        let mut invoice = match self
            .invoice_repository
            .find_open(cmd.credit_card_id)
            .await?
        {
            Some(inv) => inv,
            None => {
                let new_id = InvoiceID::from_uuid(self.id_generator.new_id());
                Invoice::new(new_id, cmd.credit_card_id, reference_month)
            }
        };

        invoice.add_purchase(purchase)?;
        self.invoice_repository.save(&invoice).await?;

        let event = PurchaseAdded {
            purchase_id,
            invoice_id: invoice.id,
            credit_card_id: cmd.credit_card_id,
            amount: cmd.total_amount,
            timestamp: chrono::Utc::now(),
        };
        self.event_publisher.publish(vec![&event]).await?;

        Ok(vec![purchase_id])
    }

    async fn handle_multi_installment(
        &self,
        cmd: RegisterPurchaseCommand,
    ) -> Result<Vec<PurchaseID>, CreditCardError> {
        let n = cmd.installments_count as i64;
        let base_amount = (cmd.total_amount.amount() / rust_decimal::Decimal::from(n)).round_dp(2);
        let remainder =
            cmd.total_amount.amount() - base_amount * rust_decimal::Decimal::from(n - 1);

        let group_id = InstallmentGroupID::from_uuid(self.id_generator.new_id());
        let start_month = YearMonth::from_date(cmd.purchased_at);
        let mut purchase_ids = Vec::with_capacity(cmd.installments_count as usize);

        for i in 0..cmd.installments_count {
            let installment_num = i + 1;
            let amount = if installment_num == cmd.installments_count {
                Money::new(remainder, cmd.total_amount.currency())
            } else {
                Money::new(base_amount, cmd.total_amount.currency())
            };

            let reference_month = add_months(&start_month, i);
            let purchase_id = PurchaseID::from_uuid(self.id_generator.new_id());

            let purchase = Purchase::new_installment(
                purchase_id,
                cmd.description.clone(),
                amount,
                cmd.installments_count,
                installment_num,
                group_id,
                cmd.category_id,
                reference_month.first_day(),
            );

            let mut invoice = match self
                .invoice_repository
                .find_by_card_and_month(cmd.credit_card_id, reference_month)
                .await?
            {
                Some(inv) => inv,
                None => {
                    let new_id = InvoiceID::from_uuid(self.id_generator.new_id());
                    Invoice::new(new_id, cmd.credit_card_id, reference_month)
                }
            };

            invoice.add_purchase(purchase)?;
            self.invoice_repository.save(&invoice).await?;

            let event = PurchaseAdded {
                purchase_id,
                invoice_id: invoice.id,
                credit_card_id: cmd.credit_card_id,
                amount,
                timestamp: chrono::Utc::now(),
            };
            self.event_publisher.publish(vec![&event]).await?;

            purchase_ids.push(purchase_id);
        }

        Ok(purchase_ids)
    }
}

fn add_months(ym: &YearMonth, months: u32) -> YearMonth {
    let total_months = ym.year * 12 + ym.month as i32 - 1 + months as i32;
    YearMonth::new(total_months / 12, (total_months % 12) as u32 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::id::{MockIdGenerator, UuidGenerator};
    use crate::shared::events::InMemoryEventDispatcher;
    use crate::shared::mock::{MockCreditCardRepository, MockInvoiceRepository};
    use crate::shared::money::Currency;

    fn setup() -> (
        Arc<MockCreditCardRepository>,
        Arc<MockInvoiceRepository>,
        Arc<InMemoryEventDispatcher>,
        Arc<MockIdGenerator>,
    ) {
        let cc_repo = Arc::new(MockCreditCardRepository::new());
        let inv_repo = Arc::new(MockInvoiceRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(MockIdGenerator::new(uuid::Uuid::new_v4()));
        (cc_repo, inv_repo, publisher, id_gen)
    }

    fn setup_multi() -> (
        Arc<MockCreditCardRepository>,
        Arc<MockInvoiceRepository>,
        Arc<InMemoryEventDispatcher>,
        Arc<UuidGenerator>,
    ) {
        let cc_repo = Arc::new(MockCreditCardRepository::new());
        let inv_repo = Arc::new(MockInvoiceRepository::new());
        let publisher = Arc::new(InMemoryEventDispatcher::new());
        let id_gen = Arc::new(UuidGenerator);
        (cc_repo, inv_repo, publisher, id_gen)
    }

    async fn make_card(cc_repo: &MockCreditCardRepository, owner: UserID, card_id: CreditCardID) {
        let card = crate::credit_card::domain::card::CreditCard::new(
            card_id,
            owner,
            "Nubank".into(),
            "Mastercard".into(),
            Money::from_cents(500000, Currency::BRL),
            20,
            27,
        );
        cc_repo.save(&card).await.unwrap();
    }

    #[tokio::test]
    async fn test_single_installment_adds_to_open_invoice() {
        let (cc_repo, inv_repo, publisher, id_gen) = setup();
        let owner = UserID::new();
        let card_id = CreditCardID::new();
        make_card(&cc_repo, owner, card_id).await;

        let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo.clone(), publisher, id_gen);

        let result = handler
            .handle(RegisterPurchaseCommand {
                owner_id: owner,
                credit_card_id: card_id,
                description: "Netflix".into(),
                total_amount: Money::from_cents(5000, Currency::BRL),
                installments_count: 1,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);

        let invoice = inv_repo.find_open(card_id).await.unwrap().unwrap();
        assert_eq!(invoice.purchases.len(), 1);
        assert_eq!(
            invoice.purchases[0].total_amount,
            Money::from_cents(5000, Currency::BRL)
        );
    }

    #[tokio::test]
    async fn test_three_installments_split_across_months() {
        let (cc_repo, inv_repo, publisher, id_gen) = setup_multi();
        let owner = UserID::new();
        let card_id = CreditCardID::new();
        make_card(&cc_repo, owner, card_id).await;

        let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo.clone(), publisher, id_gen);

        let result = handler
            .handle(RegisterPurchaseCommand {
                owner_id: owner,
                credit_card_id: card_id,
                description: "TV".into(),
                total_amount: Money::from_cents(9000, Currency::BRL),
                installments_count: 3,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 3);

        // All 3 purchases should share the same group_id
        let inv_jan = inv_repo
            .find_by_card_and_month(card_id, YearMonth::new(2026, 1))
            .await
            .unwrap()
            .unwrap();
        let inv_feb = inv_repo
            .find_by_card_and_month(card_id, YearMonth::new(2026, 2))
            .await
            .unwrap()
            .unwrap();
        let inv_mar = inv_repo
            .find_by_card_and_month(card_id, YearMonth::new(2026, 3))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(inv_jan.purchases.len(), 1);
        assert_eq!(inv_feb.purchases.len(), 1);
        assert_eq!(inv_mar.purchases.len(), 1);

        // Each installment is 3000
        assert_eq!(
            inv_jan.purchases[0].total_amount,
            Money::from_cents(3000, Currency::BRL)
        );
        assert_eq!(
            inv_feb.purchases[0].total_amount,
            Money::from_cents(3000, Currency::BRL)
        );
        assert_eq!(
            inv_mar.purchases[0].total_amount,
            Money::from_cents(3000, Currency::BRL)
        );

        // Same group_id
        let group = inv_jan.purchases[0].installment_group_id.unwrap();
        assert_eq!(inv_feb.purchases[0].installment_group_id, Some(group));
        assert_eq!(inv_mar.purchases[0].installment_group_id, Some(group));

        // Installment numbers
        assert_eq!(inv_jan.purchases[0].installment_number, 1);
        assert_eq!(inv_feb.purchases[0].installment_number, 2);
        assert_eq!(inv_mar.purchases[0].installment_number, 3);
    }

    #[tokio::test]
    async fn test_installment_remainder_goes_to_last() {
        let (cc_repo, inv_repo, publisher, id_gen) = setup_multi();
        let owner = UserID::new();
        let card_id = CreditCardID::new();
        make_card(&cc_repo, owner, card_id).await;

        let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo.clone(), publisher, id_gen);

        // 1000 / 3 = 333 * 3 = 999, remainder = 1
        let result = handler
            .handle(RegisterPurchaseCommand {
                owner_id: owner,
                credit_card_id: card_id,
                description: "Something".into(),
                total_amount: Money::from_cents(1000, Currency::BRL),
                installments_count: 3,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 3);

        let inv_jun = inv_repo
            .find_by_card_and_month(card_id, YearMonth::new(2026, 6))
            .await
            .unwrap()
            .unwrap();
        let inv_jul = inv_repo
            .find_by_card_and_month(card_id, YearMonth::new(2026, 7))
            .await
            .unwrap()
            .unwrap();
        let inv_aug = inv_repo
            .find_by_card_and_month(card_id, YearMonth::new(2026, 8))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            inv_jun.purchases[0].total_amount,
            Money::from_cents(333, Currency::BRL)
        );
        assert_eq!(
            inv_jul.purchases[0].total_amount,
            Money::from_cents(333, Currency::BRL)
        );
        assert_eq!(
            inv_aug.purchases[0].total_amount,
            Money::from_cents(334, Currency::BRL)
        );
    }

    #[tokio::test]
    async fn test_wrong_owner_fails() {
        let (cc_repo, inv_repo, publisher, id_gen) = setup();
        let owner = UserID::new();
        let card_id = CreditCardID::new();
        make_card(&cc_repo, owner, card_id).await;

        let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo, publisher, id_gen);

        let result = handler
            .handle(RegisterPurchaseCommand {
                owner_id: UserID::new(),
                credit_card_id: card_id,
                description: "Hack".into(),
                total_amount: Money::from_cents(5000, Currency::BRL),
                installments_count: 1,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            })
            .await;

        assert!(matches!(
            result,
            Err(CreditCardError::InvariantViolation(_))
        ));
    }

    #[tokio::test]
    async fn test_zero_installments_fails() {
        let (cc_repo, inv_repo, publisher, id_gen) = setup();
        let owner = UserID::new();
        let card_id = CreditCardID::new();
        make_card(&cc_repo, owner, card_id).await;

        let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo, publisher, id_gen);

        let result = handler
            .handle(RegisterPurchaseCommand {
                owner_id: owner,
                credit_card_id: card_id,
                description: "Bad".into(),
                total_amount: Money::from_cents(5000, Currency::BRL),
                installments_count: 0,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            })
            .await;

        assert!(matches!(
            result,
            Err(CreditCardError::InvariantViolation(_))
        ));
    }

    #[tokio::test]
    async fn test_installments_cross_year_boundary() {
        let (cc_repo, inv_repo, publisher, id_gen) = setup_multi();
        let owner = UserID::new();
        let card_id = CreditCardID::new();
        make_card(&cc_repo, owner, card_id).await;

        let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo.clone(), publisher, id_gen);

        let result = handler
            .handle(RegisterPurchaseCommand {
                owner_id: owner,
                credit_card_id: card_id,
                description: "Annual thing".into(),
                total_amount: Money::from_cents(12000, Currency::BRL),
                installments_count: 3,
                category_id: CategoryID::new(),
                purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 11, 10).unwrap(),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 3);

        let inv_nov = inv_repo
            .find_by_card_and_month(card_id, YearMonth::new(2026, 11))
            .await
            .unwrap()
            .unwrap();
        let inv_dec = inv_repo
            .find_by_card_and_month(card_id, YearMonth::new(2026, 12))
            .await
            .unwrap()
            .unwrap();
        let inv_jan = inv_repo
            .find_by_card_and_month(card_id, YearMonth::new(2027, 1))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            inv_nov.purchases[0].total_amount,
            Money::from_cents(4000, Currency::BRL)
        );
        assert_eq!(
            inv_dec.purchases[0].total_amount,
            Money::from_cents(4000, Currency::BRL)
        );
        assert_eq!(
            inv_jan.purchases[0].total_amount,
            Money::from_cents(4000, Currency::BRL)
        );
    }
}
