# Backlog — zxc-money (Core Library de Finanças Pessoais)

> **Arquitetura:** Lib Rust pura. Domínio modelado com DDD.
> Módulos verticais por bounded context. Comunicação via domain events.
> Persistência via ports (traits). Frontends (TUI, web, Android) são projetos separados.
> **Segurança:** Todas as operações (commands e queries) recebem `Principal` e validam ownership.

## Ordem de Execução

```
M0 (Fundação) → M1 (Ledger) → M2 (Recorrências) → M3 (CreditCard)
    → M4 (Budgeting) → M5 (Reporting) → M6 (Bills)
    → M7 (Investment) → M8 (Importação) → M9 (Planning)
    → M10 (Providers) → M11 (Polimento API)
```

- **M0-M3** = MVP funcional para qualquer frontend
- **M4-M6** = v1.0 completa
- **M7-M11** = v2.0+

## Milestones

| Milestone | Arquivo | Bounded Context | Descrição | Progresso |
|-----------|---------|-----------------|-----------|-----------|
| M0 | [milestone-00.md](milestone-00.md) | Shared + Infra | Fundação: VOs, eventos, estrutura modular | ~100% |
| M1 | [milestone-01.md](milestone-01.md) | **Ledger** | Contas, transações, transferências, conciliação | ~55% |
| M2 | [milestone-02.md](milestone-02.md) | **Ledger** | Transações recorrentes | 0% |
| M3 | [milestone-03.md](milestone-03.md) | **CreditCard** | Cartões, faturas, parcelamento | ~30% (structs) |
| M4 | [milestone-04.md](milestone-04.md) | **Budgeting** | Orçamento mensal/anual, metas financeiras | ~25% (structs) |
| M5 | [milestone-05.md](milestone-05.md) | **Reporting** | Dashboard, net worth, relatórios, exportação | ~15% (read models) |
| M6 | [milestone-06.md](milestone-06.md) | **BillsReminder** | Contas a pagar, calendário, lembretes | ~30% (structs) |
| M7 | [milestone-07.md](milestone-07.md) | **Investment** | Portfolio, ativos, rentabilidade | ~30% (structs) |
| M8 | [milestone-08.md](milestone-08.md) | **Import** | Parsers OFX/CSV, conciliação | 0% |
| M9 | [milestone-09.md](milestone-09.md) | **Planning** | Simuladores (imobiliário, aposentadoria, empréstimo, salário) | ~80% |
| M10 | [milestone-10.md](milestone-10.md) | **Provider** | Ports de infraestrutura (datetime, cotações, notificações) | ~33% |
| M11 | [milestone-11.md](milestone-11.md) | — | Polimento: docs, exemplos, publicação | ~10% |

## Documentos Relacionados

| Documento | Descrição |
|-----------|-----------|
| [spec-app-financas-pessoais.md](../spec-app-financas-pessoais.md) | Features originais (referência) |
| [ddd.md](../ddd.md) | Modelagem de domínio DDD (referência arquitetural) |
