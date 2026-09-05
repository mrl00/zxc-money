# MILESTONE 0 — Fundação e Arquitetura

**Objetivo:** Lib compila, estrutura modular vertical (bounded contexts), infra de eventos, traits de persistência, testes básicos.

## Épico 0.1 — Setup do Projeto

| ID | Task | Prioridade | Status |
|----|------|------------|--------|
| 0.1.1 | Transformar em lib crate: renomear `main.rs` → `lib.rs`, declarar módulos públicos | Alta | ✅ |
| 0.1.2 | Adicionar dependências: `serde`, `serde_json`, `chrono`, `uuid`, `thiserror`, `async-trait`, `rust_decimal`, `tokio` (dev) | Alta | ✅ |
| 0.1.3 | Configurar `rustfmt.toml` e `clippy.toml` (edição 2024) | Média | ✅ |
| 0.1.4 | Setup de CI: `cargo fmt --check`, `cargo clippy`, `cargo test` | Média | ⬜ |

## Épico 0.2 — Value Objects e Tipos Compartilhados

| ID | Task | Prioridade | Status |
|----|------|------------|--------|
| 0.2.1 | Criar `Money` (centavos i64 + Currency ISO 4217), imutável, validação de moeda em Add/Sub/Mul/Div | Alta | ✅ |
| 0.2.2 | Criar ID wrappers type-safe: `AccountID`, `TransactionID`, `CategoryID`, `TagID`, `GoalID`, etc. (cada um = newtype de Uuid) | Alta | ✅ |
| 0.2.3 | Criar `Period` (Start, End), usado por Budget e Reporting | Alta | ✅ |
| 0.2.4 | Criar `YearMonth` (usado por Invoice) | Média | ✅ |
| 0.2.5 | Criar módulo `errors.rs` com erros por módulo: `RepositoryError`, `PublishError`, `LedgerError`, `BudgetingError`, `CreditCardError`, `BillsError`, `InvestmentError` | Alta | ✅ |

## Épico 0.3 — Infraestrutura de Eventos de Domínio

| ID | Task | Prioridade | Status |
|----|------|------------|--------|
| 0.3.1 | Criar trait `DomainEvent` (event_type, timestamp, as_any) | Alta | ✅ |
| 0.3.2 | Criar trait `EventPublisher` async (publish Vec<&dyn DomainEvent>) | Alta | ✅ |
| 0.3.3 | Criar `InMemoryEventDispatcher` (dispatcher async in-process para MVP) | Alta | ✅ |
| 0.3.4 | Criar trait `EventHandler<E: DomainEvent>` | Alta | ✅ |

## Épico 0.4 — Infraestrutura de Persistência (Ports)

| ID | Task | Prioridade | Status |
|----|------|------------|--------|
| 0.4.1 | Criar trait `Repository<T, ID>` genérica com `save`, `find_by_id`, `delete` | Alta | ✅ |
| 0.4.2 | Criar trait `UnitOfWork` async (execute com closure) | Média | ✅ |
| 0.4.3 | Implementação mock de `Repository` e `UnitOfWork` para testes | Alta | ✅ |

## Épico 0.5 — Estrutura Modular (Vertical por Bounded Context)

| ID | Task | Prioridade | Status |
|----|------|------------|--------|
| 0.5.1 | Criar esqueleto de todos os bounded contexts (ledger, budgeting, credit_card, bills, investment, reporting, planning) | Alta | ✅ |
| 0.5.2 | Criar aggregates e domain events do ledger (Account, Transaction, AccountOpened, TransactionRecorded, TransferCompleted, TransactionDeleted, TransactionReconciled) | Alta | ✅ |
| 0.5.3 | Criar command handlers do ledger (OpenAccount, RecordTransaction, TransferFunds, DeleteTransaction, ReconcileTransaction) | Alta | ✅ |
| 0.5.4 | Criar aggregates dos demais contexts (Budget, Goal, CreditCard, Invoice, Bill, Portfolio, Asset) | Média | ✅ |
| 0.5.5 | Implementar Category e Tag no Ledger com repository traits | Média | ✅ |

## Épico 0.6 — Providers (Ports de Infraestrutura)

| ID | Task | Prioridade | Status |
|----|------|------------|--------|
| 0.6.1 | Criar trait `DateTimeProvider` + `SystemDateTime` + `MockDateTime` | Alta | ✅ |
| 0.6.2 | Criar trait `IdGenerator` + `UuidGenerator` + `MockIdGenerator` | Alta | ✅ |

## Épico 0.7 — Testes

| ID | Task | Prioridade | Status |
|----|------|------------|--------|
| 0.7.1 | Testes do módulo `Money` (criação, add, sub, mul, div, moeda diferente, display, serde) | Alta | ✅ |
| 0.7.2 | Testes dos ID wrappers (criação, display, igualdade, serde) | Alta | ✅ |
| 0.7.3 | Testes do `InMemoryEventDispatcher` (handler chamado, múltiplos handlers, ignorar não registrado, múltiplos eventos) | Alta | ✅ |
| 0.7.4 | Testes de `Period` e `YearMonth` (criação, contains, overlaps, ordering, serde) | Alta | ✅ |
| 0.7.5 | Testes de planning (mortgage, loan, retirement, net_salary) | Média | ✅ |
| 0.7.6 | Mock implementations de repositories para testes | Média | ✅ |
| 0.7.7 | Testes unitários de command handlers (OpenAccount, RecordTransaction, TransferFunds) | Média | ✅ |

## Épico 0.8 — Segurança

| ID | Task | Prioridade | Status |
|----|------|------------|--------|
| 0.8.1 | Tipos base: UserID, Principal, IdempotencyKey, erros Forbidden/Unauthenticated em todos os módulos | Alta | ✅ |
| 0.8.2 | Módulo identity: User aggregate, UserRepository, PasswordHasher (Argon2), CreateUser, AuthenticateUser | Alta | ✅ |
| 0.8.3 | OwnerID em todos os aggregates sensíveis (Account, Budget, FinancialGoal, CreditCard, Bill, Portfolio — excluindo Transaction por design) | Alta | ✅ |
| 0.8.4 | Authorization: todos os command handlers recebem Principal e validam ownership antes de mutar | Alta | ✅ |
| 0.8.5 | Authorization fix: query handlers recebem Principal; DefineBudget cross-user fix; owner_id em TransactionRecorded/BillScheduled/BillPaid | Alta | ✅ |
| 0.8.6 | IdempotencyKey: tipo + campo nos comandos sensíveis (RecordTransaction, TransferFunds) + IdempotencyRepository trait | Média | ✅ |
| 0.8.7 | AuditLogger: trait + AuditEntry + InMemoryAuditLogger + AuditEventHandler (buffer+flush) + AuditableEvent wrapper | Média | ✅ |
| 0.8.8 | Documentação: atualizar README.md com exemplos, seção Security, dependências novas | Baixa | ✅ |
