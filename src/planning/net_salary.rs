use serde::{Deserialize, Serialize};

use crate::shared::money::Money;

/// Brazilian tax regime for salary calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxRegime {
    /// Consolidação das Leis do Trabalho (employee).
    CLT,
    /// Pessoa Jurídica (independent contractor).
    PJ,
}

/// Breakdown of gross salary into taxes and net amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetSalaryBreakdown {
    pub gross: Money,
    pub inss: Money,
    pub irrf: Money,
    pub total_discounts: Money,
    pub net: Money,
}

/// Calculates the net salary based on gross amount, dependents, and tax regime.
///
/// # Example
/// ```ignore
/// let result = calculate_net_salary(
///     Money::new(5000, Currency::BRL),
///     0,
///     TaxRegime::CLT,
/// );
/// assert!(result.net.amount() < result.gross.amount());
/// ```
pub fn calculate_net_salary(
    gross: Money,
    dependents: u32,
    regime: TaxRegime,
) -> NetSalaryBreakdown {
    match regime {
        TaxRegime::CLT => calculate_clt(gross, dependents),
        TaxRegime::PJ => calculate_pj(gross),
    }
}

fn calculate_clt(gross: Money, dependents: u32) -> NetSalaryBreakdown {
    let gross_amount = gross.amount() as f64;

    let inss = calculate_inss(gross_amount);
    let base_irrf = gross_amount - inss - (dependents as f64 * 18959.0);
    let irrf = calculate_irrf(base_irrf);

    let total_discounts = inss + irrf;
    let net = gross_amount - total_discounts;

    NetSalaryBreakdown {
        gross,
        inss: Money::new(inss as i64, gross.currency()),
        irrf: Money::new(irrf as i64, gross.currency()),
        total_discounts: Money::new(total_discounts as i64, gross.currency()),
        net: Money::new(net as i64, gross.currency()),
    }
}

fn calculate_pj(gross: Money) -> NetSalaryBreakdown {
    let gross_amount = gross.amount() as f64;
    let das = gross_amount * 0.06;
    let net = gross_amount - das;

    NetSalaryBreakdown {
        gross,
        inss: Money::zero(gross.currency()),
        irrf: Money::zero(gross.currency()),
        total_discounts: Money::new(das as i64, gross.currency()),
        net: Money::new(net as i64, gross.currency()),
    }
}

fn calculate_inss(gross: f64) -> f64 {
    let mut tax = 0.0;

    if gross > 151800.0 {
        tax += (gross.min(279388.0) - 151800.0) * 0.075;
    }
    if gross > 279388.0 {
        tax += (gross.min(419083.0) - 279388.0) * 0.09;
    }
    if gross > 419083.0 {
        tax += (gross.min(815741.0) - 419083.0) * 0.12;
    }
    if gross > 815741.0 {
        tax += (gross - 815741.0) * 0.14;
    }

    tax
}

fn calculate_irrf(base: f64) -> f64 {
    if base <= 225920.0 {
        0.0
    } else if base <= 282665.0 {
        (base - 225920.0) * 0.075
    } else if base <= 375105.0 {
        (base - 282665.0) * 0.15 + 4256.0
    } else if base <= 466468.0 {
        (base - 375105.0) * 0.225 + 18162.0
    } else {
        (base - 466468.0) * 0.275 + 38640.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::money::Currency;

    const BRL: Currency = Currency::BRL;

    #[test]
    fn test_net_salary_clt() {
        let gross = Money::new(500000, BRL);
        let result = calculate_net_salary(gross, 0, TaxRegime::CLT);

        assert!(result.net.amount() < gross.amount());
        assert!(result.inss.amount() > 0);
        assert!(result.irrf.amount() > 0);
    }

    #[test]
    fn test_net_salary_pj() {
        let gross = Money::new(500000, BRL);
        let result = calculate_net_salary(gross, 0, TaxRegime::PJ);

        assert!(result.net.amount() < gross.amount());
        assert_eq!(result.inss.amount(), 0);
        assert_eq!(result.irrf.amount(), 0);
    }

    #[test]
    fn test_net_salary_exempt() {
        let gross = Money::new(150000, BRL);
        let result = calculate_net_salary(gross, 0, TaxRegime::CLT);

        assert_eq!(result.irrf.amount(), 0);
        assert_eq!(result.inss.amount(), 0);
    }

    #[test]
    fn test_net_salary_clt_known_values() {
        let gross = Money::new(10_000_00, BRL);
        let result = calculate_net_salary(gross, 0, TaxRegime::CLT);

        assert_eq!(result.gross.amount(), gross.amount());
        assert!(
            (result.total_discounts.amount() - (result.inss.amount() + result.irrf.amount())).abs()
                <= 1
        );
        assert!(
            (result.net.amount() - (gross.amount() - result.total_discounts.amount())).abs() <= 1
        );
        assert!(result.inss.amount() > 0);
        assert!(result.irrf.amount() > 0);
    }

    #[test]
    fn test_net_salary_pj_das_6pct() {
        let gross = Money::new(15_000_00, BRL);
        let result = calculate_net_salary(gross, 0, TaxRegime::PJ);

        assert_eq!(result.total_discounts.amount(), 900_00);
        assert_eq!(result.net.amount(), 14_100_00);
        assert_eq!(result.inss.amount(), 0);
        assert_eq!(result.irrf.amount(), 0);
    }

    #[test]
    fn test_net_salary_zero_dependents() {
        let gross = Money::new(8_000_00, BRL);
        let r0 = calculate_net_salary(gross, 0, TaxRegime::CLT);
        let r2 = calculate_net_salary(gross, 2, TaxRegime::CLT);

        assert!(r2.net.amount() > r0.net.amount());
    }

    #[test]
    fn test_net_salary_many_dependents() {
        let gross = Money::new(3_000_00, BRL);
        let result = calculate_net_salary(gross, 20, TaxRegime::CLT);

        assert!(result.net.amount() > 0);
        assert_eq!(result.irrf.amount(), 0);
    }

    #[test]
    fn test_net_salary_max_bracket() {
        let gross = Money::new(80_000_00, BRL);
        let result = calculate_net_salary(gross, 0, TaxRegime::CLT);

        assert!(result.irrf.amount() > 0);
        let irrf_rate =
            result.irrf.amount() as f64 / (gross.amount() - result.inss.amount()) as f64;
        assert!(irrf_rate > 0.25);
    }

    #[test]
    fn test_net_salary_just_below_threshold() {
        let gross = Money::new(2_259_20, BRL);
        let result = calculate_net_salary(gross, 0, TaxRegime::CLT);

        assert_eq!(result.irrf.amount(), 0);
    }

    #[test]
    fn test_net_salary_discounts_never_exceed_gross() {
        for gross_amount in [1_000_00, 5_000_00, 10_000_00, 50_000_00, 100_000_00] {
            let gross = Money::new(gross_amount, BRL);
            let result = calculate_net_salary(gross, 0, TaxRegime::CLT);
            assert!(result.net.amount() > 0);
            assert!(result.total_discounts.amount() < gross.amount());
        }
    }
}
