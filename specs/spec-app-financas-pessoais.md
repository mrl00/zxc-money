# Especificação de Features — App de Finanças Pessoais

> Baseado nas features do Mobills (referência atual de mercado) integradas com conceitos do Microsoft Money que ainda fazem sentido hoje.

## 1. Contas e Transações

- [x] Cadastro de múltiplas contas (corrente, poupança, carteira/dinheiro, investimento)
- [x] Visualização de saldo consolidado (todas as contas) e por conta individual
- [x] Registro manual de receitas e despesas
- [x] Categorias e subcategorias personalizáveis
- [x] Tags livres para classificação cruzada (ex: "viagem-2026", "reembolsável")
- [x] Transações recorrentes (assinaturas, salário, aluguel)
- [x] Transferências entre contas (sem duplicar como receita/despesa)
- [x] Conciliação bancária — marcar transação como "conferida" contra extrato (herdado do MS Money)

## 2. Cartão de Crédito

- [x] Cadastro de um ou mais cartões, com limite e dia de fechamento/vencimento
- [x] Lançamento de compras vinculado à fatura correspondente
- [x] Fechamento automático de fatura e projeção do valor total
- [x] Parcelamento de compras com exibição das parcelas futuras
- [x] Alerta de proximidade do limite

## 3. Orçamento (Budgeting)

- [x] Orçamento mensal por categoria
- [x] Comparativo orçado vs. realizado, com alerta de estouro
- [x] Orçamento anual/planejamento de longo prazo
- [x] Metas financeiras (ex: "juntar R$10.000 até dezembro"), com acompanhamento de progresso

## 4. Contas a Pagar e Lembretes

- [ ] Cadastro de contas fixas/variáveis com vencimento
- [ ] Lembrete/notificação antes do vencimento (push e/ou e-mail)
- [ ] Marcação de conta como paga, gerando a transação automaticamente
- [ ] Calendário de vencimentos

## 5. Relatórios e Visualização

- [ ] Gráficos de gastos por categoria (período customizável)
- [ ] Fluxo de caixa mensal e anual
- [ ] **Evolução de patrimônio líquido (net worth)** — conceito forte do MS Money, hoje pouco explorado pelos apps brasileiros; soma ativos (contas, investimentos) menos passivos (dívidas, cartão)
- [ ] Comparativo histórico mês a mês / ano a ano
- [ ] Exportação de relatórios (PDF, Excel/CSV, OFX)

## 6. Investimentos

- [ ] Acompanhamento de carteira (ações, fundos, renda fixa, cripto)
- [ ] Rentabilidade por ativo e da carteira consolidada
- [ ] Integração com cotações de mercado (quando aplicável)
- [ ] Inclusão dos investimentos no cálculo de patrimônio líquido (item 5)

## 7. Integração Bancária

- [ ] Conexão automática com bancos (open finance / agregador) para importação de transações
- [ ] Importação manual de extratos (OFX, CSV, PDF)
- [ ] Sincronização em nuvem entre dispositivos, com suporte a uso offline

## 8. Planejamento Financeiro (herdado do MS Money, ainda relevante)

- [x] Calculadora/simulador de financiamento imobiliário (amortização, juros)
- [x] Simulador de aposentadoria (quanto poupar por mês para atingir uma meta)
- [x] Simulador de empréstimo (baseado no perfil do usuário)
- [x] Calculadora de salário líquido

## 9. Extras e Diferenciais de Mercado

- [ ] Despesas com geolocalização (onde o gasto ocorreu)
- [ ] Armazenamento de recibos/comprovantes (foto anexada à transação)
- [ ] Divisão de despesas entre pessoas (casais, grupos) — diferencial competitivo recente
- [ ] Entrada de despesas via assistente de mensagens (WhatsApp) com IA/OCR — tendência 2026, ausente no Mobills tradicional

## 10. Fora de Escopo (features do MS Money hoje obsoletas ou de nicho)

- ~~Integração com serviço proprietário de cotações online~~ (descontinuado, hoje é padrão via API de mercado)
- ~~Exportação específica para TurboTax~~ (irrelevante fora dos EUA; substituir por exportação genérica para contador/ferramenta fiscal local)
- ~~Módulo dedicado só para impressão de cheques~~ (sem uso prático hoje)

---

*Documento gerado para uso como referência de escopo/backlog inicial de um app de finanças pessoais.*
