# MILESTONE 10 — Providers de Infraestrutura

**Objetivo:** Ports (traits) que frontends devem implementar. Sem implementação concreta no core.

## Épico 10.1 — Traits de Provider

| ID | Trait | Método | Uso no Core | Status |
|----|-------|--------|-------------|--------|
| 10.1.1 | `DateTimeProvider` | `fn now(&self) -> DateTime<Utc>` | Injected em services | ✅ |
| 10.1.2 | `IdGenerator` | `fn new_id(&self) -> Uuid` | Geração de IDs | ✅ |
| 10.1.3 | `QuoteProvider` | `async fn get_quote(ticker) -> Result<Quote>` | Cálculo de rentabilidade | ✅ |
| 10.1.4 | `NotificationProvider` | `async fn notify(title, body) -> Result<()>` | Lembretes, alertas | ✅ |
| 10.1.5 | `FileStorage` | `async fn save(path, data) -> Result<()>` | Exportação de relatórios | ✅ |
| 10.1.6 | `BankProvider` | `async fn fetch_transactions(account_id, range) -> Result<Vec<RawTransaction>>` | Importação Open Finance | ✅ |

## Épico 10.2 — Padrão de Injeção

| ID | Padrão | Exemplo | Status |
|----|--------|---------|--------|
| 10.2.1 | Services recebem providers via constructor (dependency injection) | `OpenAccountHandler::new(repo, publisher, id_gen)` | ✅ |
| 10.2.2 | Para testes: mock providers | `MockDateTime`, `MockIdGenerator`, `MockNotificationProvider`, `MockFileStorage`, `MockBankProvider` | ✅ |

## Épico 10.3 — Testes de Integração

| ID | Task | Status |
|----|------|--------|
| 10.3.1 | Testes com providers mockados | ✅ |
| 10.3.2 | Documentação de como cada frontend deve implementar cada provider | ✅ |
