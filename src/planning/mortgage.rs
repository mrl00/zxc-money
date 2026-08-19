use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::shared::money::Money;

/// Amortization method used for mortgage calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmortizationMethod {
    /// Sistema de Amortização Constante — constant principal, decreasing payments.
    SAC,
    /// Tabela Price — equal payments throughout the term.
    Price,
}

/// A single month's entry in a mortgage amortization schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmortizationEntry {
    pub month: u32,
    pub payment: Money,
    pub principal: Money,
    pub interest: Money,
    pub balance: Money,
}

/// Complete amortization schedule for a mortgage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmortizationSchedule {
    pub entries: Vec<AmortizationEntry>,
    pub total_paid: Money,
    pub total_interest: Money,
}

/// Simulates a mortgage using the chosen amortization method.
///
/// # Example
/// ```ignore
/// let schedule = simulate_mortgage(
///     Money::new(100_000, Currency::BRL),
///     12,
///     Decimal::from(12),
///     AmortizationMethod::SAC,
/// );
/// assert_eq!(schedule.entries.len(), 12);
/// ```
pub fn simulate_mortgage(
    principal: Money,
    months: u32,
    annual_rate: Decimal,
    method: AmortizationMethod,
) -> AmortizationSchedule {
    let monthly_rate = annual_rate / Decimal::from(12) / Decimal::from(100);
    let mut entries = Vec::new();
    let mut balance = principal;

    match method {
        AmortizationMethod::SAC => {
            let principal_per_month =
                Money::new(principal.amount() / months as i64, principal.currency());

            for month in 1..=months {
                let interest_amount = (balance.amount() as f64
                    * monthly_rate.to_string().parse::<f64>().unwrap())
                    as i64;
                let interest = Money::new(interest_amount, principal.currency());
                let payment = principal_per_month + interest;
                balance = Money::new(
                    balance.amount() - principal_per_month.amount(),
                    principal.currency(),
                );

                entries.push(AmortizationEntry {
                    month,
                    payment,
                    principal: principal_per_month,
                    interest,
                    balance,
                });
            }
        }
        AmortizationMethod::Price => {
            let monthly_payment_amount =
                calculate_price_payment(principal.amount(), months, monthly_rate);
            let monthly_payment = Money::new(monthly_payment_amount, principal.currency());

            for month in 1..=months {
                let interest_amount = (balance.amount() as f64
                    * monthly_rate.to_string().parse::<f64>().unwrap())
                    as i64;
                let interest = Money::new(interest_amount, principal.currency());
                let principal_part = Money::new(
                    monthly_payment.amount() - interest.amount(),
                    principal.currency(),
                );
                balance = Money::new(
                    balance.amount() - principal_part.amount(),
                    principal.currency(),
                );

                entries.push(AmortizationEntry {
                    month,
                    payment: monthly_payment,
                    principal: principal_part,
                    interest,
                    balance,
                });
            }
        }
    }

    let total_paid = Money::new(
        entries.iter().map(|e| e.payment.amount()).sum(),
        principal.currency(),
    );
    let total_interest = Money::new(
        entries.iter().map(|e| e.interest.amount()).sum(),
        principal.currency(),
    );

    AmortizationSchedule {
        entries,
        total_paid,
        total_interest,
    }
}

fn calculate_price_payment(principal: i64, months: u32, monthly_rate: Decimal) -> i64 {
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
    fn test_mortgage_sac() {
        let principal = Money::new(100000, BRL);
        let schedule = simulate_mortgage(principal, 12, Decimal::from(12), AmortizationMethod::SAC);

        assert_eq!(schedule.entries.len(), 12);
        assert!(schedule.total_interest.amount() > 0);
        assert!(schedule.total_paid.amount() > principal.amount());
        assert_eq!(
            schedule.total_paid.amount(),
            schedule
                .entries
                .iter()
                .map(|e| e.payment.amount())
                .sum::<i64>()
        );
    }

    #[test]
    fn test_mortgage_price() {
        let principal = Money::new(100000, BRL);
        let schedule =
            simulate_mortgage(principal, 12, Decimal::from(12), AmortizationMethod::Price);

        assert_eq!(schedule.entries.len(), 12);
        assert!(schedule.total_interest.amount() > 0);
        assert_eq!(
            schedule.total_paid.amount(),
            schedule
                .entries
                .iter()
                .map(|e| e.payment.amount())
                .sum::<i64>()
        );
    }

    #[test]
    fn test_mortgage_sac_known_values() {
        let principal = Money::new(120_000_00, BRL);
        let schedule =
            simulate_mortgage(principal, 240, Decimal::from(10), AmortizationMethod::SAC);

        assert_eq!(schedule.entries.len(), 240);
        let last = schedule.entries.last().unwrap();
        assert_eq!(last.balance.amount(), 0);
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
    }

    #[test]
    fn test_mortgage_price_known_values() {
        let principal = Money::new(120_000_00, BRL);
        let schedule =
            simulate_mortgage(principal, 240, Decimal::from(10), AmortizationMethod::Price);

        assert_eq!(schedule.entries.len(), 240);
        let last = schedule.entries.last().unwrap();
        assert!(last.balance.amount().abs() <= 100);
        let first_payment = schedule.entries[0].payment.amount();
        for entry in &schedule.entries {
            assert_eq!(entry.payment.amount(), first_payment);
        }
    }

    #[test]
    fn test_sac_vs_price_same_params() {
        let principal = Money::new(200_000_00, BRL);
        let months = 360;
        let rate = Decimal::from(12);

        let sac = simulate_mortgage(principal, months, rate, AmortizationMethod::SAC);
        let price = simulate_mortgage(principal, months, rate, AmortizationMethod::Price);

        assert_eq!(sac.entries.len(), price.entries.len());
        assert!(sac.total_interest.amount() < price.total_interest.amount());
        assert!(sac.entries[0].payment.amount() > price.entries[0].payment.amount());
        assert!(
            sac.entries.last().unwrap().payment.amount()
                < price.entries.last().unwrap().payment.amount()
        );
    }
}
