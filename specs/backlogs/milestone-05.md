# MILESTONE 5 — Reporting: Relatórios e Projections (Read Side)

**Objetivo:** Read models / projectors alimentados por eventos. Sem aggregates próprios.

## Épico 5.1 — Read Models / Projections

| ID | Read Model | Dados | Eventos consumidos | Status |
|----|-----------|-------|-------------------|--------|
| 5.1.1 | `AccountBalanceProjection` | Saldo atual e conciliado por Account | TransactionRecorded, TransferCompleted, TransactionDeleted | ✅ |
| 5.1.2 | `NetWorthSnapshot` | { date, total_assets, total_liabilities, net_worth } | AccountOpened, TransactionRecorded, TransferCompleted, AccountDeleted | ✅ |
| 5.1.3 | `BudgetProgress` | planejado vs realizado por Category/Period | TransactionRecorded, BudgetDefined | ✅ |
| 5.1.4 | `CashFlowSummary` | entradas/saídas agregadas por período | TransactionRecorded, TransferCompleted | ✅ |
| 5.1.5 | `CategoryReport` | gastos por categoria em período | TransactionRecorded | ✅ |

## Épico 5.2 — Application Layer (Queries)

| ID | Query | Entrada | Saída | Status |
|----|-------|---------|-------|--------|
| 5.2.1 | `GetMonthlySummary` | month, year | { total_income, total_expense, balance } | ✅ |
| 5.2.2 | `GetCashFlow` | months_back | Vec\<CashFlowEntry\> | ✅ |
| 5.2.3 | `GetNetWorthSnapshot` | date | NetWorthSnapshot | ✅ |
| 5.2.4 | `GetNetWorthHistory` | — | Vec\<NetWorthSnapshot\> | ⬜ |
| 5.2.5 | `GetNetWorthBreakdown` | — | { accounts, investments, liabilities } | ⬜ |
| 5.2.6 | `GetExpensesByCategory` | from, to | Vec\<CategoryReport\> | ✅ |
| 5.2.7 | `GetTopExpenses` | from, to, limit | Vec\<Transaction\> | ✅ |
| 5.2.8 | `GetMonthComparison` | months: Vec\<(u32,u32)\> | Vec\<MonthSummary\> | ⬜ |
| 5.2.9 | `GetYearComparison` | years: Vec\<u32\> | Vec\<YearSummary\> | ⬜ |

## Épico 5.3 — Exportação

| ID | Query | Formato | Status |
|----|-------|---------|--------|
| 5.3.1 | `ExportTransactions` | CSV (via `csv` crate, writer) | ⬜ |
| 5.3.2 | `ExportData` | JSON (via `serde_json`, writer) | ⬜ |
