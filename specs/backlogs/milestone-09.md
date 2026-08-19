# MILESTONE 9 — Planning: Simuladores Financeiros

**Objetivo:** Domain services puros (stateless), sem aggregate, sem repositório. Funções determinísticas.

## Épico 9.1 — Financiamento Imobiliário

| ID | Função | Assinatura | Status |
|----|--------|-----------|--------|
| 9.1.1 | `simulate_mortgage` | `fn(principal, months, annual_rate, method) -> AmortizationSchedule` | ✅ |
| 9.1.2 | `AmortizationMethod` | enum `SAC \| Price` | ✅ |
| 9.1.3 | `AmortizationSchedule` | struct `{ entries, total_paid, total_interest }` | ✅ |
| 9.1.4 | `AmortizationEntry` | struct `{ month, payment, principal, interest, balance }` | ✅ |

## Épico 9.2 — Simulador de Aposentadoria

| ID | Função | Assinatura | Status |
|----|--------|-----------|--------|
| 9.2.1 | `simulate_retirement` | `fn(current_savings, monthly_contribution, years, annual_return) -> RetirementProjection` | ✅ |
| 9.2.2 | `required_contribution` | `fn(target, current_savings, years, annual_return) -> Money` | ✅ |
| 9.2.3 | `RetirementProjection` | struct `{ timeline, final_balance }` | ✅ |

## Épico 9.3 — Simulador de Empréstimo

| ID | Função | Assinatura | Status |
|----|--------|-----------|--------|
| 9.3.1 | `simulate_loan` | `fn(principal, months, annual_rate) -> LoanSchedule` | ✅ |
| 9.3.2 | `LoanSchedule` | struct `{ entries, total_paid, total_interest }` | ✅ |

## Épico 9.4 — Salário Líquido

| ID | Função | Assinatura | Status |
|----|--------|-----------|--------|
| 9.4.1 | `calculate_net_salary` | `fn(gross, dependents, regime) -> NetSalaryBreakdown` | ✅ |
| 9.4.2 | `NetSalaryBreakdown` | struct `{ gross, inss, irrf, total_discounts, net }` | ✅ |
| 9.4.3 | Tabelas INSS/IRRF | Hardcoded ou configurável | ✅ |

## Épico 9.5 — Testes

| ID | Task | Status |
|----|------|--------|
| 9.5.1 | Testes unitários de cada simulador com valores conhecidos | ✅ |
| 9.5.2 | Comparação SAC vs Price com mesmos parâmetros | ✅ |
| 9.5.3 | Cálculo de salário líquido com casos de borda (isento, faixa máxima) | ✅ |
