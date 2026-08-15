# zxc-money

A core library for personal finance management, written in Rust.

## What is it?

zxc-money is a headless backend for managing personal finances. It handles accounts, transactions, budgets, credit cards, bills, investments, and financial planning — all without being tied to any specific UI.

## What is it for?

Build any personal finance app on top of it: a terminal TUI, a web app, a mobile app, or all of them. The core logic lives here; each frontend project consumes this library independently.

## Features

- **Accounts & Transactions** — manage multiple accounts, record income/expenses, transfer between accounts
- **Budgeting** — set monthly/annual budgets per category, track planned vs actual spending
- **Goals** — create financial goals with target amounts and deadlines
- **Credit Cards** — register cards, track invoices, handle installments
- **Bills** — schedule recurring bills, get reminded before due dates
- **Investments** — track assets, calculate profitability per holding and portfolio-wide
- **Reporting** — net worth snapshots, cash flow summaries, category breakdowns
- **Financial Planning** — mortgage simulator, retirement planner, loan calculator, net salary calculator

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
zxc-money = { path = "../zxc-money" }
```

Example — create an account and a transaction:

```rust
use zxc_money::shared::money::{Money, Currency};
use zxc_money::shared::ids::AccountID;

// Create a money value
let balance = Money::new(100_00, Currency::BRL); // R$ 100.00

// Generate a new account ID
let account_id = AccountID::new();
```

Example — use a financial simulator:

```rust
use zxc_money::planning::mortgage::simulate_mortgage;
use zxc_money::shared::money::{Money, Currency};
use zxc_money::shared::period::AmortizationMethod;

let schedule = simulate_mortgage(
    Money::new(500_000_00, Currency::BRL), // R$ 500,000.00
    360, // 30 years
    rust_decimal_macros::dec!(0.10), // 10% annual rate
    AmortizationMethod::SAC,
);

println!("Total paid: {:?}", schedule.total_paid);
```

## License

[MIT](LICENSE)
