use serde::{Deserialize, Serialize};

use crate::shared::money::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionEntry {
    pub year: u32,
    pub balance: Money,
    pub contributions: Money,
    pub growth: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetirementProjection {
    pub timeline: Vec<ProjectionEntry>,
    pub final_balance: Money,
}

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
            let growth_amount = (balance.amount() as f64 * monthly_return) as i64;
            let growth = Money::new(growth_amount, balance.currency());
            balance = balance + growth;
            year_growth = year_growth + growth;

            balance = balance + monthly_contribution;
            year_contributions = year_contributions + monthly_contribution;
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
        current_savings.amount() as f64 * (1.0 + monthly_return).powf(months as f64);
    let remaining = target.amount() as f64 - future_value_of_current;

    if remaining <= 0.0 {
        return Money::zero(target.currency());
    }

    let monthly_contribution =
        remaining * monthly_return / ((1.0 + monthly_return).powf(months as f64) - 1.0);

    Money::new(monthly_contribution as i64, target.currency())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retirement_simulation() {
        let current = Money::new(100000, crate::shared::money::Currency::BRL);
        let monthly = Money::new(1000, crate::shared::money::Currency::BRL);
        let projection = simulate_retirement(current, monthly, 10, rust_decimal::Decimal::from(8));

        assert_eq!(projection.timeline.len(), 10);
        assert!(projection.final_balance.amount() > current.amount());
    }
}
