# Ducker Git Workflow

Este documento define como o Git deve ser utilizado durante o desenvolvimento do Ducker.

O objetivo principal é evitar que uma mudança experimental ou uma alteração sensível destrua uma versão que já funciona.

---

## 1. Regra principal

> **Cada mensagem/tarefa concluída deve resultar em um commit.**

Uma mensagem representa uma unidade de trabalho.

Exemplo:

```text
Mensagem 1
→ implementar descoberta
→ testar
→ commit

Mensagem 2
→ adicionar seleção de dispositivo
→ testar
→ commit

Mensagem 3
→ implementar envio
→ testar
→ commit
```

Não acumular várias tarefas independentes em um único commit.

Isso cria um histórico que permite descobrir exatamente qual mudança causou um problema.

---

## 2. O commit deve representar um estado funcional

Sempre que possível, o agente deve:

1. entender a tarefa;
2. alterar o código;
3. executar os testes relevantes;
4. verificar se o projeto continua funcionando;
5. criar o commit.

O commit deve representar uma alteração deliberada e identificável.

Evitar commits com código obviamente quebrado apenas para "salvar progresso".

---

## 3. Mensagens de commit

As mensagens devem ser claras e objetivas.

Exemplos:

```text
feat: add device discovery
feat: add quac device identity
feat: implement file transfer
fix: reject invalid destination quac id
test: add transfer integration test
docs: update transfer protocol
refactor: separate discovery service
```

O formato recomendado é:

```text
<tipo>: <descrição>
```

Tipos comuns:

- `feat` → nova funcionalidade
- `fix` → correção
- `refactor` → reorganização sem mudança intencional de comportamento
- `test` → testes
- `docs` → documentação
- `build` → sistema de build/dependências
- `chore` → manutenção

---

# 4. Mudanças sensíveis

Nem toda mudança deve ser feita diretamente no branch principal.

Se uma alteração puder quebrar partes importantes do sistema, o agente deve considerar a mudança **sensível**.

Exemplos:

- alteração do protocolo de transferência;
- mudança no formato das mensagens entre dispositivos;
- alteração do sistema de descoberta;
- mudança no ID Quac;
- mudança no networking;
- alteração do Core Rust;
- mudança de APIs públicas;
- alteração de persistência;
- troca de dependências fundamentais;
- mudanças que afetam simultaneamente CLI e Mobile;
- alterações de segurança;
- refatorações grandes.

---

# 5. Branch para mudanças sensíveis

Para uma mudança sensível, criar um branch separado antes de implementar.

Exemplo:

```bash
git switch -c feature/transfer-protocol-v2
```

ou:

```bash
git switch -c experiment/new-discovery
```

A nomenclatura deve explicar o objetivo.

Exemplos:

```text
feature/device-discovery
feature/file-transfer
feature/mobile-receiver

experiment/new-protocol
experiment/discovery-rewrite

fix/quac-validation
refactor/core-networking
```

---

# 6. Por que usar branch?

O branch funciona como uma área de teste.

A ideia é:

```text
                 main
                  │
                  │ versão estável
                  │
                  ▼
             ┌─────────┐
             │ mudança │
             │ sensível│
             └────┬────┘
                  │
                  ▼
          branch experimental
             /           \
          funciona      quebra
             │             │
             ▼             ▼
          merge          apagar/
          no main        voltar
```

Se funcionar, a mudança pode ser integrada.

Se quebrar, a base estável continua intacta.

---

# 7. Não usar branch para tudo

Branches não devem virar burocracia.

Mudanças pequenas e isoladas podem ser feitas diretamente no branch principal quando não houver risco relevante.

Exemplos:

- corrigir typo;
- atualizar documentação;
- ajustar mensagem da CLI;
- adicionar teste simples;
- corrigir erro pequeno e localizado.

Regra prática:

> **Se a mudança puder quebrar a arquitetura, o protocolo ou múltiplos componentes, use branch.**

---

# 8. Commits dentro de um branch sensível

Mesmo em um branch experimental, manter commits pequenos.

Exemplo:

```text
main
  │
  └── feature/new-transfer-protocol
          │
          ├── commit 1: add protocol types
          ├── commit 2: add handshake
          ├── commit 3: update receiver
          ├── commit 4: update CLI
          └── commit 5: add integration tests
```

Isso permite voltar para qualquer etapa.

---

# 9. Testar antes de integrar

Uma mudança sensível só deve voltar para `main` depois de passar pelos testes relevantes.

Fluxo:

```text
main
  ↓
criar branch
  ↓
implementar
  ↓
testar
  ↓
corrigir
  ↓
testar novamente
  ↓
validar CLI/Mobile quando aplicável
  ↓
merge
  ↓
main
```

---

# 10. Se der errado

Se uma mudança experimental quebrar o projeto:

**não tentar consertar tudo diretamente no `main`.**

Primeiro preservar o estado estável.

Dependendo da situação:

```bash
git switch main
```

O branch experimental pode continuar existindo para investigação ou ser descartado se não tiver utilidade.

Se o branch tiver commits úteis, eles podem ser reaproveitados seletivamente.

---

# 11. Rollback

Quando uma mudança já integrada quebrar algo, preferir mecanismos que preservem o histórico.

Para desfazer uma mudança já publicada:

```bash
git revert <commit>
```

Evitar reescrever o histórico compartilhado sem necessidade.

`reset --hard` pode ser utilizado em branches locais/experimentais quando apropriado, mas não deve ser usado casualmente para apagar histórico que outras pessoas já estejam utilizando.

---

# 12. Nunca trabalhar sem histórico

O problema que originou esta regra foi a perda do projeto durante uma formatação sem nenhum commit.

Portanto:

> **O Git faz parte do sistema de segurança do desenvolvimento do Ducker.**

Depois que uma unidade de trabalho estiver funcional:

```text
código
 ↓
teste
 ↓
commit
```

Não deixar várias funcionalidades prontas apenas no estado local sem commit.

---

# 13. Estado estável

O branch principal (`main`) deve representar, tanto quanto possível, um estado conhecido e utilizável.

Idealmente:

```text
main
│
├── commit A  ✓
├── commit B  ✓
├── commit C  ✓
└── commit D  ✓
```

Não utilizar `main` como laboratório para mudanças arquiteturais experimentais.

---

# 14. Regra para agentes de IA

Agentes trabalhando no Ducker devem seguir este comportamento:

### Antes da alteração

Avaliar:

- a mudança é pequena?
- afeta um único componente?
- pode quebrar protocolo?
- pode quebrar Core?
- afeta CLI e Mobile?
- muda API pública?
- muda persistência?
- muda segurança?

### Se baixo risco

Pode trabalhar no branch atual.

### Se alto risco

Criar branch específico antes de alterar o código.

### Depois da alteração

- testar;
- verificar regressões;
- criar commit;
- informar o commit realizado.

---

# 15. O agente não deve fazer

O agente não deve:

- acumular várias tarefas sem commit;
- alterar `main` com uma grande refatoração experimental;
- apagar histórico para esconder uma alteração;
- fazer `force push` sem autorização;
- misturar uma correção não relacionada na mesma tarefa;
- marcar uma mudança como concluída sem testar quando houver teste disponível;
- deixar uma alteração arquitetural grande sem branch quando houver risco relevante.

---

# 16. Fluxo oficial do Ducker

```text
┌──────────────────────┐
│ Receber nova tarefa  │
└──────────┬───────────┘
           ↓
┌──────────────────────┐
│ Avaliar risco        │
└──────────┬───────────┘
           │
      ┌────┴────┐
      │         │
    baixo      alto
      │         │
      ↓         ↓
    main      branch
      │         │
      └────┬────┘
           ↓
    implementar
           ↓
       testar
           ↓
    ┌──────┴──────┐
    │             │
   falha         passa
    │             │
    ↓             ↓
 corrigir      commit
                  ↓
           se branch: validar
                  ↓
              merge
                  ↓
                main
```

---

# 17. Regra resumida

Se for necessário lembrar apenas cinco coisas:

1. **Uma tarefa concluída = um commit.**
2. **Commit deve representar uma alteração clara.**
3. **Mudança sensível = branch separado.**
4. **Branch experimental que funcionar pode voltar para `main`.**
5. **Se quebrar, a versão estável continua preservada.**

O objetivo não é ter um Git bonito.

O objetivo é poder dizer:

> "Essa alteração quebrou o Ducker? Beleza. Qual commit foi? Reverte."

E continuar trabalhando. 🦆
