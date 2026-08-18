# MILESTONE 3 — CreditCard: Cartão, Faturas e Parcelamento

**Objetivo:** Aggregate CreditCard e Invoice, entity Purchase, parcelamento, integração com Ledger via eventos.

## Épico 3.1 — Aggregate: CreditCard

| ID | User Story | Tasks Técnicas | Status |
|----|-----------|----------------|--------|
| 3.1.1 | Como frontend, quero cadastrar cartão com nome, bandeira, limite, dia fechamento/vencimento | `CreditCard` aggregate: `id`, `name`, `brand`, `limit`, `closing_day`, `due_day` | ✅ |
| 3.1.2 | Como frontend, quero ver disponível e utilizado | Read model `CreditCardSummary` | ✅ |

## Épico 3.2 — Aggregate: Invoice

| ID | User Story | Tasks Técnicas | Status |
|----|-----------|----------------|--------|
| 3.2.1 | Como frontend, quero ver fatura aberta | `Invoice` aggregate com `status` (Open/Closed/Paid) | ✅ |
| 3.2.2 | Como frontend, quero fechar fatura | Command `CloseInvoice`, publica `InvoiceClosed` | ✅ |
| 3.2.3 | Como frontend, quero pagar fatura | Command `PayInvoice`, publica `InvoicePaid` → Ledger cria Transaction | ✅ |

## Épico 3.3 — Entity: Purchase

| ID | User Story | Tasks Técnicas | Status |
|----|-----------|----------------|--------|
| 3.3.1 | Como frontend, quero registrar compra no cartão | `Purchase` entity, adicionada à Invoice Open | ✅ |
| 3.3.2 | Como frontend, quero parcelar compra em N vezes | `Purchase` replicada em múltiplas Invoices via `InstallmentGroupID`, com remainder na última parcela | ✅ |

## Épico 3.4 — Alertas

| ID | User Story | Tasks Técnicas | Status |
|----|-----------|----------------|--------|
| 3.4.1 | Como frontend, quero saber se cartão atingiu % do limite | `CreditCardService::check_limit_alert()` | ✅ |

## Épico 3.5 — Domain Events

| ID | Evento | Consumido por | Status |
|----|--------|---------------|--------|
| 3.5.1 | `PurchaseAdded` | Reporting | ✅ |
| 3.5.2 | `InvoiceClosed` | Reporting | ✅ |
| 3.5.3 | `InvoicePaid` | **Ledger** (gera Transaction expense) | ✅ |
