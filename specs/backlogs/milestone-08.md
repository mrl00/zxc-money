# MILESTONE 8 — Importação de Dados

**Objetivo:** Parsers de extratos e conciliação. Ports para parsers (implementação no frontend).

## Épico 8.1 — Ports de Parser

| ID | Port | Método | Status |
|----|------|--------|--------|
| 8.1.1 | `StatementParser` | `fn parse(reader, format) -> Result<Vec<RawTransaction>>` — implementação no frontend | ✅ |
| 8.1.2 | `ColumnMapping` | Configuração de mapeamento de colunas CSV | ✅ |

## Épico 8.2 — Domain Model: RawTransaction

| ID | Campo | Tipo | Status |
|----|-------|------|--------|
| 8.2.1 | `date` | Date | ✅ |
| 8.2.2 | `amount` | Money | ✅ |
| 8.2.3 | `description` | String | ✅ |
| 8.2.4 | `raw_line` | String (original) | ✅ |

## Épico 8.3 — Application Service: ImportService

| ID | User Story | Tasks Técnicas | Status |
|----|-----------|----------------|--------|
| 8.3.1 | Como frontend, quero revisar antes de importar | `ImportService::preview()` com flag `duplicate` | ✅ |
| 8.3.2 | Como frontend, quero conciliar com existentes | `ImportService::match_candidates()` — matching por data + valor aproximado | ✅ |
| 8.3.3 | Como frontend, quero confirmar importação | `ImportService::confirm()` — cria Transactions | ✅ |

## Épico 8.4 — Port: BankProvider

| ID | Port | Método | Status |
|----|------|--------|--------|
| 8.4.1 | `BankProvider` | `async fn fetch_transactions(account_id, date_range) -> Result<Vec<RawTransaction>>` | ✅ (implementado no M10) |
