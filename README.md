# zxc-money

A pure-Rust personal finance core library built with Domain-Driven Design (DDD). Designed as a modular monolith consumed by any frontend (TUI, web, mobile) via well-defined ports/traits — the core knows nothing about HTTP, terminals, or UI frameworks.

## Architecture

```
zxc-money (lib crate)
├── shared/          → Value objects, error types, event system, ID types, mocks
├── provider/        → Port traits for ID generation and datetime
├── ledger/          → Accounts, transactions, transfers, recurring transactions
├── credit_card/     → Credit cards, invoices, purchases, installments
├── bills/           → Bills reminder (scheduled payments)
├── budgeting/       → Budgets by category, financial goals
├── investment/      → Portfolios, assets, positions
├── planning/        → Loan, mortgage, retirement, net salary simulators
└── reporting/       → Account balance projections, net worth (read side)
```

Each module follows DDD layered architecture:

```
module/
├── domain/
│   ├── mod.rs
│   ├── {aggregate}.rs     → Aggregate root / entity
│   ├── events.rs          → Domain events
│   └── repository.rs      → Repository traits (ports)
└── application/
    ├── mod.rs
    └── {use_case}.rs       → Command handler (one per operation)
```

**Key principles:**
- All money amounts stored as `i64` cents (no floating-point)
- Cross-module communication via domain events (never direct repository access)
- Errors are exhaustive enums (`thiserror`) — frontends use `match`, no string parsing
- Generic handlers with dependency injection (`Arc<R>`, `Arc<P>`, `Arc<I>`)

## Quick Start

### Add dependency

```toml
[dependencies]
zxc-money = { path = "../zxc-money" }
```

### Create an Account

```rust
use zxc_money::ledger::application::open_account::{OpenAccountCommand, OpenAccountHandler};
use zxc_money::ledger::domain::account::AccountType;
use zxc_money::provider::id::UuidGenerator;
use zxc_money::shared::events::InMemoryEventDispatcher;
use zxc_money::shared::ids::UserID;
use zxc_money::shared::money::{Currency, Money};
use zxc_money::shared::mock::MockAccountRepository;

#[tokio::main]
async fn main() {
    let repo = std::sync::Arc::new(MockAccountRepository::new());
    let publisher = std::sync::Arc::new(InMemoryEventDispatcher::new());
    let id_gen = std::sync::Arc::new(UuidGenerator);

    let handler = OpenAccountHandler::new(repo, publisher, id_gen);

    let account_id = handler.handle(OpenAccountCommand {
        owner_id: UserID::new(),
        name: "Nubank Checking".into(),
        account_type: AccountType::Checking,
        currency: Currency::BRL,
        opening_balance: Money::new(150_00, Currency::BRL), // R$ 150.00
    }).await.unwrap();

    println!("Account created: {}", account_id);
}
```

### Record a Transaction

```rust
use zxc_money::ledger::application::record_transaction::{
    RecordTransactionCommand, RecordTransactionHandler,
};
use zxc_money::ledger::domain::transaction::TransactionType;
use zxc_money::shared::ids::{AccountID, CategoryID};

let handler = RecordTransactionHandler::new(repo, tx_repo, publisher, id_gen);

let tx_id = handler.handle(RecordTransactionCommand {
    account_id,
    tx_type: TransactionType::Expense,
    amount: Money::new(49_90, Currency::BRL), // R$ 49.90
    description: "Netflix".into(),
    date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
    category_id: Some(CategoryID::new()),
}).await.unwrap();
```

### Transfer Between Accounts

```rust
use zxc_money::ledger::application::transfer_funds::{
    TransferFundsCommand, TransferFundsHandler,
};

let handler = TransferFundsHandler::new(account_repo, tx_repo, publisher, id_gen);

handler.handle(TransferFundsCommand {
    from_account_id: checking_id,
    to_account_id: savings_id,
    amount: Money::new(500_00, Currency::BRL), // R$ 500.00
    description: "Monthly savings".into(),
    date: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
}).await.unwrap();
```

### Register a Credit Card Purchase

```rust
use zxc_money::credit_card::application::register_purchase::{
    RegisterPurchaseCommand, RegisterPurchaseHandler,
};

let handler = RegisterPurchaseHandler::new(cc_repo, inv_repo, publisher, id_gen);

let purchase_id = handler.handle(RegisterPurchaseCommand {
    owner_id,
    credit_card_id: card_id,
    description: "Amazon".into(),
    total_amount: Money::new(299_90, Currency::BRL), // R$ 299.90
    installments_count: 3,
    category_id: CategoryID::new(),
    purchased_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
}).await.unwrap();
```

### Close & Pay an Invoice

```rust
use zxc_money::credit_card::application::close_invoice::{
    CloseInvoiceCommand, CloseInvoiceHandler,
};
use zxc_money::credit_card::application::pay_invoice::{
    PayInvoiceCommand, PayInvoiceHandler,
};

// Close (typically at closing_day)
let close_handler = CloseInvoiceHandler::new(cc_repo, inv_repo.clone(), publisher.clone());
let invoice_id = close_handler.handle(CloseInvoiceCommand {
    owner_id,
    credit_card_id: card_id,
}).await.unwrap();

// Pay (typically at due_day)
let pay_handler = PayInvoiceHandler::new(cc_repo, inv_repo, publisher);
pay_handler.handle(PayInvoiceCommand {
    owner_id,
    credit_card_id: card_id,
    invoice_id,
}).await.unwrap();
// → This publishes InvoicePaid, which the Ledger picks up to create an Expense transaction
```

### Check Credit Card Limit

```rust
use zxc_money::credit_card::application::check_limit::CreditCardService;

let service = CreditCardService::new(inv_repo);

let summary = service.summary(&card).await.unwrap();
println!("Used: {}, Available: {} ({:.1}%)",
    summary.used, summary.available, summary.utilization_pct);

if let Some(alert) = service.check_limit_alert(&card, 80.0).await.unwrap() {
    println!("Warning: credit card limit alert!");
}
```

### Set Up Recurring Transactions

```rust
use zxc_money::ledger::application::create_recurring::{
    CreateRecurringTransactionCommand, CreateRecurringTransactionHandler,
};
use zxc_money::ledger::domain::recurring_transaction::Frequency;

let handler = CreateRecurringTransactionHandler::new(recurring_repo, publisher, id_gen);

let recurring_id = handler.handle(CreateRecurringTransactionCommand {
    owner_id,
    account_id,
    tx_type: TransactionType::Expense,
    amount: Money::new(39_90, Currency::BRL),
    description: "Netflix".into(),
    category_id: Some(CategoryID::new()),
    frequency: Frequency::Monthly,
    next_date: chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
}).await.unwrap();
```

### Generate & Confirm Pending Recurrences

```rust
use zxc_money::ledger::application::generate_pending::GeneratePendingRecurringQuery;
use zxc_money::ledger::application::confirm_recurring::{
    ConfirmRecurringCommand, ConfirmRecurringHandler,
};

// Find what's due today
let query = GeneratePendingRecurringQuery::new(recurring_repo.clone());
let pending = query.execute(chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()).await?;
// → Returns Vec of PendingRecurring items

// Confirm (creates Transaction, advances next_date)
let confirm = ConfirmRecurringHandler::new(recurring_repo, tx_repo, publisher, id_gen);
let tx_id = confirm.handle(ConfirmRecurringCommand {
    recurring_transaction_id: pending[0].recurring_transaction_id,
}).await?;
```

### Simulate a Mortgage

```rust
use zxc_money::planning::mortgage::{simulate_mortgage, AmortizationMethod};

let schedule = simulate_mortgage(
    Money::new(500_000_00, Currency::BRL), // R$ 500,000.00
    360,                                    // 30 years
    rust_decimal::Decimal::from(12),        // 12% annual
    AmortizationMethod::SAC,
);

println!("Total paid: {}", schedule.total_paid);
println!("Total interest: {}", schedule.total_interest);
for entry in &schedule.entries {
    println!("Month {}: payment={}, principal={}, interest={}, balance={}",
        entry.month, entry.payment, entry.principal, entry.interest, entry.balance);
}
```

### Calculate Net Salary

```rust
use zxc_money::planning::net_salary::{calculate_net_salary, TaxRegime};

let result = calculate_net_salary(
    Money::new(10_000_00, Currency::BRL), // R$ 10,000.00 gross
    1,                                     // 1 dependent
    TaxRegime::CLT,
);

println!("Gross:  {}", result.gross);
println!("INSS:   {}", result.inss);
println!("IRRF:   {}", result.irrf);
println!("Net:    {}", result.net);
```

## Domain Model

### Ledger

| Aggregate | Key Fields | Invariants |
|-----------|-----------|------------|
| **Account** | id, owner_id, name, type, currency, opening_balance | name non-empty, currency matches balance |
| **Transaction** | id, account_id, type, amount, description, date, category_id, tags | amount > 0, transfers require counterpart, income/expense require category |
| **RecurringTransaction** | id, owner_id, account_id, type, amount, frequency, next_date, active | same as Transaction, cannot be Transfer |

**Transfer** creates two linked `Transaction` records atomically (source + destination).

**RecurringTransaction frequency variants:** `Daily`, `Weekly`, `Biweekly`, `Monthly`, `Quarterly`, `Yearly`. The `advance()` method calculates the next occurrence, clamping to valid day-of-month.

### Credit Card

| Aggregate | Key Fields | Invariants |
|-----------|-----------|------------|
| **CreditCard** | id, owner_id, name, brand, limit, closing_day, due_day | — |
| **Invoice** | id, credit_card_id, reference_month, purchases[], status | Open→Closed→Paid state machine |
| **Purchase** | id, description, total_amount, installments_count, category_id, purchased_at | — |

**Invoice state machine:** `Open` → `Close()` → `Closed` → `Pay()` → `Paid`. Purchases can only be added to Open invoices. Payment publishes `InvoicePaid` event → Ledger creates Expense transaction.

**Installment calculation:** `installment_amount = total_amount / installments_count` (integer division, truncates).

### Bills Reminder

| Aggregate | Key Fields | Invariants |
|-----------|-----------|------------|
| **Bill** | id, owner_id, name, amount (Option), due_date, recurrence, category_id, status | Pending→Paid or Pending→Overdue |

**Recurrence rules:** `Monthly`, `Weekly`, `Yearly`. `next_due_date()` calculates the next occurrence.

### Budgeting

| Aggregate | Key Fields | Invariants |
|-----------|-----------|------------|
| **Budget** | id, owner_id, category_id, period, planned_amount | — |
| **FinancialGoal** | id, owner_id, name, target_amount, current_amount, target_date, status | can only contribute to InProgress goals, auto-achieves when current ≥ target |

### Investment

| Aggregate | Key Fields | Invariants |
|-----------|-----------|------------|
| **Portfolio** | id, owner_id, positions[] | weighted average cost on buy, validates quantity on sell |
| **Asset** | id, ticker, name, class, currency | — |

**Asset classes:** `Stock`, `Fund`, `FixedIncome`, `Crypto`. Portfolio calculates profit on sell (proceeds - cost basis).

### Planning (Stateless)

Pure calculation functions, no aggregates or persistence:

| Function | Description |
|----------|-------------|
| `simulate_loan(principal, months, rate)` | French amortization schedule |
| `simulate_mortgage(principal, months, rate, SAC\|Price)` | SAC (constant principal) or Price (constant payment) |
| `simulate_retirement(savings, monthly, years, return)` | Year-by-year projection |
| `required_contribution(target, savings, years, return)` | Monthly savings needed |
| `calculate_net_salary(gross, dependents, CLT\|PJ)` | Brazilian INSS/IRRF breakdown |

## Event System

Domain events implement the `DomainEvent` trait and are published via `EventPublisher`:

```rust
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, events: Vec<&dyn DomainEvent>) -> Result<(), PublishError>;
}
```

**Cross-context event flow:**

```
CreditCard.InvoicePaid ──event──▶ Ledger (creates Expense transaction)
                                  Reporting (updates account balance projection)
```

**In-memory dispatcher** (for testing and single-process use):

```rust
use zxc_money::shared::events::InMemoryEventDispatcher;

let dispatcher = InMemoryEventDispatcher::new();
dispatcher.register_handler("InvoicePaid", |event| {
    // handle event
});
dispatcher.publish(vec![&event]).await?;
```

## Error Handling

Every module defines its own error enum. Handlers return `Result<T, ModuleError>`:

```rust
// Ledger errors
match result {
    Err(LedgerError::AccountNotFound(id)) => { /* ... */ }
    Err(LedgerError::InsufficientFunds { available, requested }) => { /* ... */ }
    Err(LedgerError::CurrencyMismatch { expected, received }) => { /* ... */ }
    Err(LedgerError::InvariantViolation(msg)) => { /* ... */ }
    Err(LedgerError::Forbidden(msg)) => { /* ... */ }
    Err(LedgerError::Repository(e)) => { /* ... */ }
    Ok(id) => { /* ... */ }
}

// Credit card errors
match result {
    Err(CreditCardError::CreditCardNotFound(id)) => { /* ... */ }
    Err(CreditCardError::InvoiceNotFound(id)) => { /* ... */ }
    Err(CreditCardError::InvoiceNotOpen) => { /* ... */ }
    Err(CreditCardError::InvariantViolation(msg)) => { /* ... */ }
    Ok(()) => { /* ... */ }
}
```

## ID Types

14 type-safe ID wrappers (UUID v4) via `define_id!` macro:

| Type | Module |
|------|--------|
| `AccountID`, `TransactionID`, `CategoryID`, `TagID` | Ledger |
| `RecurringTransactionID` | Ledger |
| `CreditCardID`, `InvoiceID`, `PurchaseID` | CreditCard |
| `BillID` | Bills |
| `BudgetID`, `GoalID` | Budgeting |
| `AssetID`, `PortfolioID` | Investment |
| `UserID` | Shared (owner_id on all sensitive aggregates) |

## Testing

The library provides in-memory mock repositories for all module repositories:

```rust
use zxc_money::shared::mock::{
    MockAccountRepository,
    MockTransactionRepository,
    MockRecurringTransactionRepository,
    MockCreditCardRepository,
    MockInvoiceRepository,
};

// All mocks implement their respective repository traits
// and use Mutex<HashMap> for thread-safe in-memory storage
```

Run all tests:

```bash
cargo test
```

## Integration Guide

Frontends consume `zxc-money` through **facades** (one per bounded context) and implement **provider traits** for infrastructure.

### 1. Add dependency

```toml
[dependencies]
zxc-money = { path = "../zxc-money" }
```

### 2. Implement provider traits

```rust
use zxc_money::provider::{DateTimeProvider, IdGenerator};

struct MyDateTime;
impl DateTimeProvider for MyDateTime {
    fn now(&self) -> chrono::DateTime<chrono::Utc> { chrono::Utc::now() }
}

struct MyIdGen;
impl IdGenerator for MyIdGen {
    fn new_id(&self) -> uuid::Uuid { uuid::Uuid::new_v4() }
}
```

### 3. Create a facade

```rust
use std::sync::Arc;
use zxc_money::LedgerFacade;
use zxc_money::shared::mock::MockAccountRepository;
use zxc_money::shared::events::InMemoryEventDispatcher;

let accounts = Arc::new(MockAccountRepository::new());
let events = Arc::new(InMemoryEventDispatcher::new());
let ids = Arc::new(MyIdGen);

let facade = LedgerFacade::new(
    accounts,
    /* transaction_repo */ todo!(),
    /* recurring_repo */ todo!(),
    events,
    ids,
);
```

### 4. Use facade methods

```rust
use zxc_money::ledger::application::open_account::OpenAccountCommand;

let account_id = facade.open_account(OpenAccountCommand {
    owner_id: user_id,
    name: "My Account".into(),
    account_type: AccountType::Checking,
    currency: Currency::BRL,
    opening_balance: Money::new(0, Currency::BRL),
}).await?;
```

### Available provider traits

| Trait | Purpose | Implement with |
|-------|---------|---------------|
| `DateTimeProvider` | Current time | `SystemDateTime` (default) |
| `IdGenerator` | UUID generation | `UuidGenerator` (default) |
| `NotificationProvider` | Push/email alerts | Firebase, OneSignal, SMTP |
| `FileStorage` | File I/O for exports | Local filesystem, S3 |
| `BankProvider` | Open Finance APIs | Pluggy, Belvo, custom |

### Available facades

| Facade | Bounded Context |
|--------|----------------|
| `LedgerFacade` | Accounts, transactions, transfers, recurring |
| `CreditCardFacade` | Cards, invoices, purchases |
| `BillsFacade` | Bill reminders |
| `BudgetingFacade` | Budgets, financial goals |
| `InvestmentFacade` | Portfolios, assets |
| `ImportingFacade` | Statement import pipeline |
| `ReportingFacade` | Balance projections, net worth |

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `serde` / `serde_json` | Serialization |
| `chrono` | Date/time types |
| `uuid` | UUID generation (v4) |
| `thiserror` | Error derive macros |
| `async-trait` | Async trait support |
| `rust_decimal` | Precise decimal arithmetic |

## Milestones

| # | Milestone | Status |
|---|-----------|--------|
| M0 | Crate setup, shared types, event system | ✅ |
| M1 | Ledger core (accounts, transactions, transfers, reconciliation) | ✅ |
| M2 | Recurring transactions | ✅ |
| M3 | Credit card (cards, invoices, purchases, installments) | ✅ |
| M4 | Bills reminder | ✅ |
| M5 | Budgeting & financial goals | ✅ |
| M6 | Investment portfolio | ✅ |
| M7 | Reporting projections | ✅ |
| M8 | Cross-module event wiring + statement import | ✅ |
| M9 | Planning simulators | ✅ |
| M10 | Facade / public API + provider ports | ✅ |
| M11 | Documentation, examples, integration guide | ✅ |
