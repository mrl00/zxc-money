use std::sync::Arc;

use crate::reporting::projections::net_worth::NetWorthStore;
use crate::shared::ids::{AccountID, PortfolioID};
use crate::shared::money::Money;

/// Breakdown of net worth by component type.
#[derive(Debug, Clone)]
pub struct NetWorthBreakdown {
    /// Per-account balances.
    pub accounts: Vec<(AccountID, Money)>,
    /// Per-portfolio cost bases.
    pub investments: Vec<(PortfolioID, Money)>,
    /// Total across all accounts.
    pub total_accounts: Money,
    /// Total across all investments.
    pub total_investments: Money,
}

/// Query to get a detailed breakdown of net worth.
pub struct GetNetWorthBreakdownQuery;

/// Handles [`GetNetWorthBreakdownQuery`] by reading from the [`NetWorthStore`].
pub struct GetNetWorthBreakdownHandler {
    net_worth_store: Arc<NetWorthStore>,
}

impl GetNetWorthBreakdownHandler {
    /// Creates a new handler with the given net worth store.
    pub fn new(net_worth_store: Arc<NetWorthStore>) -> Self {
        Self { net_worth_store }
    }

    /// Returns the net worth breakdown.
    pub fn handle(&self, _query: GetNetWorthBreakdownQuery) -> NetWorthBreakdown {
        let accounts = self.net_worth_store.get_all_balances();
        let investments = self.net_worth_store.get_all_investment_balances();

        let total_accounts: Money = accounts.iter().fold(
            Money::zero(crate::shared::money::Currency::BRL),
            |acc, &(_, m)| acc + m,
        );

        let total_investments: Money = investments.iter().fold(
            Money::zero(crate::shared::money::Currency::BRL),
            |acc, &(_, m)| acc + m,
        );

        NetWorthBreakdown {
            accounts,
            investments,
            total_accounts,
            total_investments,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::investment::domain::events::AssetBought;
    use crate::ledger::domain::events::AccountOpened;
    use crate::shared::ids::{AssetID, UserID};
    use crate::shared::money::{Currency, Money};

    fn brl(amount: i64) -> Money {
        Money::new(amount, Currency::BRL)
    }

    #[test]
    fn test_breakdown_with_accounts() {
        let store = Arc::new(NetWorthStore::new());
        let handler = GetNetWorthBreakdownHandler::new(store.clone());

        let a1 = AccountID::new();
        let a2 = AccountID::new();

        store.handle_event(&AccountOpened {
            account_id: a1,
            owner_id: UserID::new(),
            name: "Checking".into(),
            currency: Currency::BRL,
            opening_balance: brl(5000_00),
            timestamp: chrono::Utc::now(),
        });

        store.handle_event(&AccountOpened {
            account_id: a2,
            owner_id: UserID::new(),
            name: "Savings".into(),
            currency: Currency::BRL,
            opening_balance: brl(10000_00),
            timestamp: chrono::Utc::now(),
        });

        let breakdown = handler.handle(GetNetWorthBreakdownQuery);
        assert_eq!(breakdown.accounts.len(), 2);
        assert_eq!(breakdown.total_accounts.amount(), 15000_00);
        assert!(breakdown.investments.is_empty());
    }

    #[test]
    fn test_breakdown_with_investments() {
        let store = Arc::new(NetWorthStore::new());
        let handler = GetNetWorthBreakdownHandler::new(store.clone());

        let pid = PortfolioID::new();
        store.handle_event(&AssetBought {
            portfolio_id: pid,
            owner_id: UserID::new(),
            asset_id: AssetID::new(),
            quantity: rust_decimal::Decimal::from(10),
            price: brl(2500),
            timestamp: chrono::Utc::now(),
        });

        let breakdown = handler.handle(GetNetWorthBreakdownQuery);
        assert!(breakdown.accounts.is_empty());
        assert_eq!(breakdown.investments.len(), 1);
        assert_eq!(breakdown.total_investments.amount(), 250_00);
    }

    #[test]
    fn test_breakdown_empty() {
        let store = Arc::new(NetWorthStore::new());
        let handler = GetNetWorthBreakdownHandler::new(store);

        let breakdown = handler.handle(GetNetWorthBreakdownQuery);
        assert!(breakdown.accounts.is_empty());
        assert!(breakdown.investments.is_empty());
        assert!(breakdown.total_accounts.is_zero());
        assert!(breakdown.total_investments.is_zero());
    }
}
