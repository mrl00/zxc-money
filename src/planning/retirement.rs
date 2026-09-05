use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Serialize};

use crate::shared::money::Money;

/// Yearly snapshot in a retirement projection timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionEntry {
    pub year: u32,
    pub balance: Money,
    pub contributions: Money,
    pub growth: Money,
}

/// Result of a retirement savings simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetirementProjection {
    pub timeline: Vec<ProjectionEntry>,
    pub final_balance: Money,
}

/// Projects retirement savings growth over time.
///
/// # Example
/// ```ignore
/// let projection = simulate_retirement(
///     Money::from_cents(100000, Currency::BRL),
///     Money::from_cents(1000, Currency::BRL),
///     10,
///     Decimal::from(8),
/// );
/// assert_eq!(projection.timeline.len(), 10);
/// ```
pub fn simulate_retirement(
    current_savings: Money,
    monthly_contribution: Money,
    years: u32,
    annual_return: rust_decimal::Decimal,
) -> RetirementProjection {
    let mut timeline = Vec::new();
    let mut balance = current_savings;
    let annual_return_f64 = annual_return.to_string().parse::<f64>().unwrap() / 100.0;
    let monthly_return = (1.0 + annual_return_f64).powf(1.0 / 12.0) - 1.0;

    for year in 1..=years {
        let mut year_contributions = Money::zero(current_savings.currency());
        let mut year_growth = Money::zero(current_savings.currency());

        for _ in 0..12 {
            let growth_amount =
                Decimal::try_from(balance.amount().to_f64().unwrap() * monthly_return).unwrap();
            let growth = Money::new(growth_amount, balance.currency());
            balance = (balance + growth).unwrap();
            year_growth = (year_growth + growth).unwrap();

            balance = (balance + monthly_contribution).unwrap();
            year_contributions = (year_contributions + monthly_contribution).unwrap();
        }

        timeline.push(ProjectionEntry {
            year,
            balance,
            contributions: year_contributions,
            growth: year_growth,
        });
    }

    RetirementProjection {
        timeline,
        final_balance: balance,
    }
}

/// Calculates the monthly contribution needed to reach a retirement target.
///
/// Returns zero if current savings already meet or exceed the target.
pub fn required_contribution(
    target: Money,
    current_savings: Money,
    years: u32,
    annual_return: rust_decimal::Decimal,
) -> Money {
    let annual_return_f64 = annual_return.to_string().parse::<f64>().unwrap() / 100.0;
    let monthly_return = (1.0 + annual_return_f64).powf(1.0 / 12.0) - 1.0;
    let months = years * 12;

    let future_value_of_current =
        current_savings.amount().to_f64().unwrap() * (1.0 + monthly_return).powf(months as f64);
    let remaining = target.amount().to_f64().unwrap() - future_value_of_current;

    if remaining <= 0.0 {
        return Money::zero(target.currency());
    }

    let monthly_contribution =
        remaining * monthly_return / ((1.0 + monthly_return).powf(months as f64) - 1.0);

    Money::new(
        Decimal::try_from(monthly_contribution).unwrap(),
        target.currency(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::Currency;

    const BRL: Currency = Currency::BRL;

    #[test]
    fn test_retirement_simulation() {
        let current = Money::from_cents(100000, BRL);
        let monthly = Money::from_cents(1000, BRL);
        let projection = simulate_retirement(current, monthly, 10, Decimal::from(8));

        assert_eq!(projection.timeline.len(), 10);
        assert!(projection.final_balance.amount() > current.amount());
    }

    #[test]
    fn test_retirement_known_values() {
        let current = Money::from_cents(10000000, BRL);
        let monthly = Money::from_cents(200000, BRL);
        let projection = simulate_retirement(current, monthly, 5, Decimal::from(10));

        assert_eq!(projection.timeline.len(), 5);
        let total_contributions = projection
            .timeline
            .iter()
            .map(|e| e.contributions.amount())
            .sum::<Decimal>();
        assert_eq!(
            total_contributions,
            monthly.amount() * Decimal::from(12 * 5)
        );
        assert!(projection.final_balance.amount() > current.amount() + total_contributions);
    }

    #[test]
    fn test_retirement_zero_return() {
        let current = Money::from_cents(5000000, BRL);
        let monthly = Money::from_cents(100000, BRL);
        let projection = simulate_retirement(current, monthly, 10, rust_decimal::Decimal::ZERO);

        assert_eq!(
            projection.final_balance.amount(),
            current.amount() + monthly.amount() * Decimal::from(120)
        );
    }

    #[test]
    fn test_retirement_higher_return_gives_more() {
        let current = Money::from_cents(10000000, BRL);
        let monthly = Money::from_cents(200000, BRL);
        let low = simulate_retirement(current, monthly, 10, Decimal::from(5));
        let high = simulate_retirement(current, monthly, 10, Decimal::from(12));

        assert!(high.final_balance.amount() > low.final_balance.amount());
    }

    #[test]
    fn test_required_contribution_known_values() {
        let target = Money::from_cents(100000000, BRL);
        let current = Money::from_cents(20000000, BRL);
        let result = required_contribution(target, current, 20, Decimal::from(8));

        assert!(result.amount() > Decimal::ZERO);
        let projection = simulate_retirement(current, result, 20, Decimal::from(8));
        assert!(projection.final_balance.amount() >= target.amount() - Decimal::from(100));
    }

    #[test]
    fn test_required_contribution_already_reached() {
        let target = Money::from_cents(10000000, BRL);
        let current = Money::from_cents(20000000, BRL);
        let result = required_contribution(target, current, 10, Decimal::from(8));

        assert_eq!(result.amount(), Decimal::ZERO);
    }
}
