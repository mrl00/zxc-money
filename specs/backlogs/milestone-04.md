# MILESTONE 4 — Budgeting: Orçamento e Metas

**Objetivo:** Aggregate Budget e FinancialGoal, read model BudgetProgress, integração com Ledger via eventos.

## Épico 4.1 — Aggregate: Budget

| ID | User Story | Tasks Técnicas | Status |
|----|-----------|----------------|--------|
| 4.1.1 | Como frontend, quero definir orçamento mensal por categoria | `Budget` aggregate com invariante: um único Budget por (category_id, period). Command `DefineBudget` | ✅ |
| 4.1.2 | Como frontend, quero ver orçado vs realizado | Read model `BudgetProgress` derivado de eventos | ✅ |
| 4.1.3 | Como frontend, quero alerta de estouro | `BudgetProgress::is_over` calculado pelo projector | ✅ |

## Épico 4.2 — Orçamento Anual

| ID | User Story | Tasks Técnicas | Status |
|----|-----------|----------------|--------|
| 4.2.1 | Como frontend, quero definir metas anuais | `BudgetService::create_annual()` — cria 12 Budgets | ✅ |
| 4.2.2 | Como frontend, quero evolução mensal do orçamento anual | Read model `AnnualBudgetProgress` | ✅ |

## Épico 4.3 — Aggregate: FinancialGoal

| ID | User Story | Tasks Técnicas | Status |
|----|-----------|----------------|--------|
| 4.3.1 | Como frontend, quero criar meta financeira | `FinancialGoal` aggregate com status (InProgress/Achieved/Abandoned) | ✅ |
| 4.3.2 | Como frontend, quero ver progresso da meta | `GoalProgress` { pct_complete, remaining, monthly_needed } | ✅ |
| 4.3.3 | Como frontend, quero contribuir para meta | Command `ContributeToGoal`, publica `GoalContributed` / `GoalAchieved` | ✅ |
| 4.3.4 | Como frontend, quero vincular meta a conta | `Goal` reage a `TransactionRecorded` para atualizar `current_amount` | ✅ |

## Épico 4.4 — Domain Events

| ID | Evento | Consumido por | Status |
|----|--------|---------------|--------|
| 4.4.1 | `BudgetDefined` | Reporting | ✅ |
| 4.4.2 | `BudgetExceeded` | Reporting / Notificações | ✅ |
| 4.4.3 | `GoalContributed` | Reporting | ✅ |
| 4.4.4 | `GoalAchieved` | Reporting / Notificações | ✅ |

## Épico 4.5 — Integração com Ledger

| ID | Integração | Mecanismo | Status |
|----|-----------|-----------|--------|
| 4.5.1 | TransactionRecorded → atualiza BudgetProgress | Event handler no Budgeting projector | ✅ |
| 4.5.2 | TransactionRecorded → atualiza Goal.current_amount (se vinculada) | Event handler no Budgeting module | ✅ |
