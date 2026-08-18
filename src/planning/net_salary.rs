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

    #[test]
    fn test_net_salary_clt() {
        let gross = Money::new(500000, crate::shared::money::Currency::BRL);
        let result = calculate_net_salary(gross, 0, TaxRegime::CLT);

        assert!(result.net.amount() < gross.amount());
        assert!(result.inss.amount() > 0);
        assert!(result.irrf.amount() > 0);
    }

    #[test]
    fn test_net_salary_pj() {
        let gross = Money::new(500000, crate::shared::money::Currency::BRL);
        let result = calculate_net_salary(gross, 0, TaxRegime::PJ);

        assert!(result.net.amount() < gross.amount());
        assert!(result.inss.amount() == 0);
        assert!(result.irrf.amount() == 0);
    }

    #[test]
    fn test_net_salary_exempt() {
        let gross = Money::new(150000, crate::shared::money::Currency::BRL);
        let result = calculate_net_salary(gross, 0, TaxRegime::CLT);

        assert_eq!(result.irrf.amount(), 0);
        assert_eq!(result.inss.amount(), 0);
    }
}
