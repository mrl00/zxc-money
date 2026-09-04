use serde::{Deserialize, Serialize};

use crate::shared::money::Money;

/// A single month's entry in a loan amortization schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanEntry {
    pub month: u32,
    pub payment: Money,
    pub principal: Money,
    pub interest: Money,
    pub balance: Money,
}

/// Complete amortization schedule for a loan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanSchedule {
    pub entries: Vec<LoanEntry>,
    pub total_paid: Money,
    pub total_interest: Money,
}

/// Simulates a fixed-rate loan with equal monthly payments (French amortization).
///
/// # Example
/// ```ignore
/// let schedule = simulate_loan(
///     Money::new(50_000, Currency::BRL),
///     12,
///     Decimal::from(10),
/// );
/// assert_eq!(schedule.entries.len(), 12);
/// ```
pub fn simulate_loan(
    principal: Money,
    months: u32,
    annual_rate: rust_decimal::Decimal,
) -> LoanSchedule {
    let monthly_rate =
        annual_rate / rust_decimal::Decimal::from(12) / rust_decimal::Decimal::from(100);
    let mut entries = Vec::new();
    let mut balance = principal;

    let monthly_payment_amount = calculate_loan_payment(principal.amount(), months, monthly_rate);
    let monthly_payment = Money::new(monthly_payment_amount, principal.currency());

    for month in 1..=months {
        let interest_amount =
            (balance.amount() as f64 * monthly_rate.to_string().parse::<f64>().unwrap()) as i64;
        let interest = Money::new(interest_amount, principal.currency());
        let principal_part = Money::new(
            monthly_payment.amount() - interest.amount(),
            principal.currency(),
        );
        balance = Money::new(
            balance.amount() - principal_part.amount(),
            principal.currency(),
        );

        entries.push(LoanEntry {
            month,
            payment: monthly_payment,
            principal: principal_part,
            interest,
            balance,
        });
    }

    let total_paid = Money::new(
        entries.iter().map(|e| e.payment.amount()).sum(),
        principal.currency(),
    );
    let total_interest = Money::new(
        entries.iter().map(|e| e.interest.amount()).sum(),
        principal.currency(),
    );

    LoanSchedule {
        entries,
        total_paid,
        total_interest,
    }
}

fn calculate_loan_payment(principal: i64, months: u32, monthly_rate: rust_decimal::Decimal) -> i64 {
    let r = monthly_rate.to_string().parse::<f64>().unwrap();
    let n = months as f64;
    let p = principal as f64;
    let payment = p * (r * (1.0 + r).powf(n)) / ((1.0 + r).powf(n) - 1.0);
    payment as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::Currency;

    const BRL: Currency = Currency::BRL;

    #[test]
    fn test_loan_simulation() {
        let principal = Money::new(50000, BRL);
        let schedule = simulate_loan(principal, 12, rust_decimal::Decimal::from(10));

        assert_eq!(schedule.entries.len(), 12);
        assert!(schedule.total_interest.amount() > 0);
    }

    #[test]
    fn test_loan_known_values() {
        let principal = Money::new(50_000_00, BRL);
        let schedule = simulate_loan(principal, 24, rust_decimal::Decimal::from(12));

        assert_eq!(schedule.entries.len(), 24);
        assert_eq!(
            schedule.total_paid.amount(),
            schedule
                .entries
                .iter()
                .map(|e| e.payment.amount())
                .sum::<i64>()
        );
        assert_eq!(
            schedule.total_interest.amount(),
            schedule
                .entries
                .iter()
                .map(|e| e.interest.amount())
                .sum::<i64>()
        );
        let last = schedule.entries.last().unwrap();
        assert!(last.balance.amount().abs() <= 100);
        assert!(
            (schedule.total_paid.amount()
                - (schedule.total_interest.amount() + principal.amount()))
            .abs()
                <= 100
        );
    }

    #[test]
    fn test_loan_all_payments_equal() {
        let principal = Money::new(100_000_00, BRL);
        let schedule = simulate_loan(principal, 60, rust_decimal::Decimal::from(8));

        let first = schedule.entries[0].payment.amount();
        for entry in &schedule.entries {
            assert_eq!(entry.payment.amount(), first);
        }
    }
}
