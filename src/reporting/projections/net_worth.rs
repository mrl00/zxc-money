use std::collections::HashMap;
use std::sync::Mutex;

use crate::investment::domain::events::{AssetBought, AssetSold};
use crate::ledger::domain::events::{
    AccountDeleted, AccountOpened, TransactionRecorded, TransferCompleted,
};
use crate::ledger::domain::transaction::TransactionType;
use crate::reporting::projections::account_balance::NetWorthSnapshot;
use crate::shared::events::DomainEvent;
use crate::shared::ids::{AccountID, PortfolioID};
use crate::shared::money::Money;

/// In-memory store that maintains net worth snapshots from domain events.
///
/// Tracks both bank account balances and investment portfolio cost bases.
/// Investment balances reflect cost basis (total invested), not market value.
/// To obtain market value, use
/// [`GetPortfolioSummaryHandler`](crate::investment::application::get_portfolio_summary::GetPortfolioSummaryHandler)
/// with live prices.
pub struct NetWorthStore {
    balances: Mutex<HashMap<AccountID, Money>>,
    investment_balances: Mutex<HashMap<PortfolioID, Money>>,
}

impl NetWorthStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self {
            balances: Mutex::new(HashMap::new()),
            investment_balances: Mutex::new(HashMap::new()),
        }
    }

    /// Returns a net worth snapshot for the current state.
    ///
    /// Includes both bank account balances and investment cost bases.
    pub fn snapshot(&self, date: chrono::NaiveDate) -> NetWorthSnapshot {
        let balances = self.balances.lock().unwrap();
        let total_accounts: Money = balances.values().fold(
            Money::zero(crate::shared::money::Currency::BRL),
            |acc, &m| (acc + m).unwrap(),
        );

        let investment_balances = self.investment_balances.lock().unwrap();
        let total_investments: Money = investment_balances.values().fold(
            Money::zero(crate::shared::money::Currency::BRL),
            |acc, &m| (acc + m).unwrap(),
        );

        let total_assets = (total_accounts + total_investments).unwrap();

        NetWorthSnapshot {
            date,
            total_assets,
            total_liabilities: Money::zero(crate::shared::money::Currency::BRL),
            net_worth: total_assets,
        }
    }

    /// Returns the balance for a specific account, if tracked.
    pub fn get_balance(&self, account_id: AccountID) -> Option<Money> {
        let balances = self.balances.lock().unwrap();
        balances.get(&account_id).copied()
    }

    /// Returns all account balances.
    pub fn get_all_balances(&self) -> Vec<(AccountID, Money)> {
        let balances = self.balances.lock().unwrap();
        balances.iter().map(|(&id, &amount)| (id, amount)).collect()
    }

    /// Returns the investment cost basis for a specific portfolio, if tracked.
    pub fn get_investment_balance(&self, portfolio_id: PortfolioID) -> Option<Money> {
        let investment_balances = self.investment_balances.lock().unwrap();
        investment_balances.get(&portfolio_id).copied()
    }

    /// Returns all investment portfolio cost bases.
    pub fn get_all_investment_balances(&self) -> Vec<(PortfolioID, Money)> {
        let investment_balances = self.investment_balances.lock().unwrap();
        investment_balances
            .iter()
            .map(|(&id, &amount)| (id, amount))
            .collect()
    }

    /// Applies a domain event to update the net worth projection.
    pub fn handle_event(&self, event: &dyn DomainEvent) {
        if let Some(e) = event.as_any().downcast_ref::<AccountOpened>() {
            let mut balances = self.balances.lock().unwrap();
            balances.insert(e.account_id, e.opening_balance);
        }

        if let Some(e) = event.as_any().downcast_ref::<TransactionRecorded>() {
            let mut balances = self.balances.lock().unwrap();
            if let Some(balance) = balances.get_mut(&e.account_id) {
                match e.tx_type {
                    TransactionType::Income => {
                        *balance = (*balance + e.amount).unwrap();
                    }
                    TransactionType::Expense => {
                        *balance = (*balance - e.amount).unwrap();
                    }
                    TransactionType::Transfer => {
                        // Transfers don't change net worth (money moves between accounts)
                    }
                }
            }
        }

        if let Some(e) = event.as_any().downcast_ref::<TransferCompleted>() {
            let mut balances = self.balances.lock().unwrap();
            if let Some(from) = balances.get_mut(&e.from_account_id) {
                *from = (*from - e.amount).unwrap();
            }
            if let Some(to) = balances.get_mut(&e.to_account_id) {
                *to = (*to + e.amount).unwrap();
            }
        }

        if let Some(e) = event.as_any().downcast_ref::<AccountDeleted>() {
            let mut balances = self.balances.lock().unwrap();
            balances.remove(&e.account_id);
        }

        // Investment events: track cost basis
        if let Some(e) = event.as_any().downcast_ref::<AssetBought>() {
            let mut investment_balances = self.investment_balances.lock().unwrap();
            let cost = e.price * e.quantity;
            let entry = investment_balances
                .entry(e.portfolio_id)
                .or_insert_with(|| Money::zero(e.price.currency()));
            *entry = (*entry + cost).unwrap();
        }

        if let Some(e) = event.as_any().downcast_ref::<AssetSold>() {
            let mut investment_balances = self.investment_balances.lock().unwrap();
            if let Some(balance) = investment_balances.get_mut(&e.portfolio_id) {
                // Subtract cost basis at sale price × quantity
                let proceeds = e.price * e.quantity;
                *balance = (*balance - proceeds).unwrap();
            }
        }
    }
}

impl Default for NetWorthStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::investment::domain::events::{AssetBought, AssetSold};
    use crate::ledger::domain::events::{AccountOpened, TransactionRecorded};
    use crate::shared::ids::{CategoryID, TransactionID, UserID};
    use crate::shared::money::{Currency, Money};

    fn brl(amount: i64) -> Money {
        Money::from_cents(amount, Currency::BRL)
    }

    fn cents(val: i64) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(val) / rust_decimal::Decimal::from(100)
    }

    fn open_account(store: &NetWorthStore, id: AccountID, balance: i64) {
        store.handle_event(&AccountOpened {
            account_id: id,
            owner_id: UserID::new(),
            name: "Test".into(),
            currency: Currency::BRL,
            opening_balance: Money::from_cents(balance, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });
    }

    #[test]
    fn test_net_worth_single_account() {
        let store = NetWorthStore::new();
        let id = AccountID::new();
        open_account(&store, id, 1000_00);

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(snapshot.total_assets.amount(), cents(1000_00));
        assert_eq!(snapshot.net_worth.amount(), cents(1000_00));
    }

    #[test]
    fn test_net_worth_multiple_accounts() {
        let store = NetWorthStore::new();
        let id1 = AccountID::new();
        let id2 = AccountID::new();
        open_account(&store, id1, 5000_00);
        open_account(&store, id2, 3000_00);

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(snapshot.total_assets.amount(), cents(8000_00));
    }

    #[test]
    fn test_net_worth_after_income() {
        let store = NetWorthStore::new();
        let id = AccountID::new();
        open_account(&store, id, 1000_00);

        store.handle_event(&TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: id,
            tx_type: TransactionType::Income,
            amount: Money::from_cents(500_00, Currency::BRL),
            category_id: Some(CategoryID::new()),
            description: "Salary".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
        assert_eq!(snapshot.total_assets.amount(), cents(1500_00));
    }

    #[test]
    fn test_net_worth_after_expense() {
        let store = NetWorthStore::new();
        let id = AccountID::new();
        open_account(&store, id, 1000_00);

        store.handle_event(&TransactionRecorded {
            transaction_id: TransactionID::new(),
            account_id: id,
            tx_type: TransactionType::Expense,
            amount: Money::from_cents(200_00, Currency::BRL),
            category_id: Some(CategoryID::new()),
            description: "Food".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
        assert_eq!(snapshot.total_assets.amount(), cents(800_00));
    }

    #[test]
    fn test_net_worth_transfer_unchanged() {
        let store = NetWorthStore::new();
        let from_id = AccountID::new();
        let to_id = AccountID::new();
        open_account(&store, from_id, 1000_00);
        open_account(&store, to_id, 500_00);

        store.handle_event(&TransferCompleted {
            from_account_id: from_id,
            to_account_id: to_id,
            amount: Money::from_cents(300_00, Currency::BRL),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(snapshot.total_assets.amount(), cents(1500_00));
        assert_eq!(snapshot.net_worth.amount(), cents(1500_00));
    }

    #[test]
    fn test_net_worth_account_deleted() {
        let store = NetWorthStore::new();
        let id = AccountID::new();
        open_account(&store, id, 1000_00);

        store.handle_event(&AccountDeleted {
            account_id: id,
            owner_id: UserID::new(),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(snapshot.total_assets.amount(), cents(0));
    }

    #[test]
    fn test_net_worth_with_asset_bought() {
        let store = NetWorthStore::new();
        let pid = PortfolioID::new();
        let aid = crate::shared::ids::AssetID::new();

        store.handle_event(&AssetBought {
            portfolio_id: pid,
            owner_id: UserID::new(),
            asset_id: aid,
            quantity: rust_decimal::Decimal::from(10),
            price: brl(2500),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        // 10 × R$25.00 = R$250.00
        assert_eq!(snapshot.total_assets.amount(), cents(250_00));
    }

    #[test]
    fn test_net_worth_with_asset_sold() {
        let store = NetWorthStore::new();
        let pid = PortfolioID::new();
        let aid = crate::shared::ids::AssetID::new();

        // Buy 10 at R$25.00
        store.handle_event(&AssetBought {
            portfolio_id: pid,
            owner_id: UserID::new(),
            asset_id: aid,
            quantity: rust_decimal::Decimal::from(10),
            price: brl(2500),
            timestamp: chrono::Utc::now(),
        });

        // Sell 5 at R$30.00 (cost basis: 5 × R$25.00 = R$125.00)
        store.handle_event(&AssetSold {
            portfolio_id: pid,
            owner_id: UserID::new(),
            asset_id: aid,
            quantity: rust_decimal::Decimal::from(5),
            price: brl(3000),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        // 250 - 150 = 100 (remaining: 5 × R$25.00 = R$125.00 cost basis at buy price)
        // Actually: buy adds 250, sell subtracts 5*3000=15000 cents = 150
        // So: 25000 - 15000 = 10000 cents = R$100.00
        assert_eq!(snapshot.total_assets.amount(), cents(100_00));
    }

    #[test]
    fn test_net_worth_accounts_plus_investments() {
        let store = NetWorthStore::new();
        let account_id = AccountID::new();
        let pid = PortfolioID::new();
        let aid = crate::shared::ids::AssetID::new();

        open_account(&store, account_id, 1000_00);

        store.handle_event(&AssetBought {
            portfolio_id: pid,
            owner_id: UserID::new(),
            asset_id: aid,
            quantity: rust_decimal::Decimal::from(10),
            price: brl(2500),
            timestamp: chrono::Utc::now(),
        });

        let snapshot = store.snapshot(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        // Accounts: R$1000.00 + Investments: R$250.00 = R$1250.00
        assert_eq!(snapshot.total_assets.amount(), cents(1250_00));
        assert_eq!(snapshot.net_worth.amount(), cents(1250_00));
    }
}
