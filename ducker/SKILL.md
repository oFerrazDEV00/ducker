---
name: ducker
description: Skill exclusiva para desenvolvimento do Ducker, um projeto open source local-first de transferência de arquivos pela rede. Use esta skill sempre que trabalhar no Ducker, incluindo arquitetura, Rust Core, CLI, Mobile, descoberta de dispositivos, ID Quac, protocolo de transferência, testes, documentação e roadmap.
---

# Ducker Skill

## Identidade do projeto

**Nome:** Ducker  
**Slogan:** Be simple, be duck.  
**Natureza:** open source  
**Objetivo:** transferência de arquivos pela rede, com foco local-first.

O Ducker não deve depender de cloud para realizar sua função principal.

## Regra mais importante

> Faça primeiro funcionar. Aprofunde depois.

O MVP atual existe para provar uma coisa:

**um dispositivo consegue encontrar outro pela rede e transferir um arquivo para o destinatário correto.**

Não bloquear o MVP por recursos avançados.

## Escopo atual do MVP

Prioridade absoluta:

1. CLI
2. Mobile
3. descoberta de dispositivos pela rede
4. identidade por ID Quac
5. associação entre nome e ID Quac
6. seleção de destinatário
7. transferência de arquivo
8. validação do ID Quac de destino
9. rejeição de destino incorreto
10. testes CLI ↔ Mobile

### Fora do MVP

Não implementar como requisito da primeira transferência:

- `.quack`
- Teams
- criptografia avançada
- MCP
- n8n
- cloud
- contas centralizadas
- permissões complexas
- recursos sociais

Esses recursos podem ser planejados, mas não devem bloquear a transferência básica.

# Arquitetura

O projeto deve buscar um núcleo compartilhado, preferencialmente em Rust.

```text
                    DUCKER
                       |
                 Ducker Core
                    (Rust)
                       |
          +------------+------------+
          |                         |
         CLI                      Mobile
          |                         |
          +------------+------------+
                       |
                    Rede local
```

O Core deve concentrar a lógica compartilhável:

- identidade
- ID Quac
- descoberta
- conexão
- protocolo
- transferência
- validação
- estados
- erros

A interface não deve duplicar a lógica de networking sem necessidade.

# Identidade

Cada instalação do Ducker possui:

- `device_name`
- `quac_id`

Fluxo inicial:

```text
Nome do dispositivo
        ↓
geração do ID Quac
        ↓
persistência local
        ↓
anúncio na rede
```

Exemplo:

```text
Nome: Celular do Fael
ID Quac: 12345678
```

O ID Quac identifica o destinatário dentro do sistema de descoberta.

O nome é a informação amigável mostrada na interface. O ID continua sendo usado para validação.

## Descoberta

Um dispositivo deve conseguir encontrar outros dispositivos Ducker na mesma rede.

O resultado da descoberta deve conter, no mínimo:

```text
quac_id
device_name
endereço necessário para conexão
porta
versão do protocolo
```

A UI deve mostrar algo como:

```text
Dispositivos encontrados

1. Celular do Fael
   Quac: 12345678

2. Notebook
   Quac: 84726193
```

O usuário escolhe o dispositivo pelo nome, enquanto o sistema usa o ID Quac para identificar e validar o destino.

# Transferência

Fluxo mínimo:

```text
1. descobrir dispositivos
2. usuário escolhe destinatário
3. abrir sessão
4. informar ID Quac de destino
5. destinatário valida o ID
6. enviar metadados
7. transferir bytes
8. confirmar conclusão
```

## Validação obrigatória

O receptor deve verificar:

```text
destination_quac_id == own_quac_id
```

Se for diferente:

```text
REJECT
DESTINATION_ID_MISMATCH
```

O arquivo **não deve ser salvo**.

Mensagem amigável:

> Ocorreu um erro: a transferência foi interceptada porque o ID Quac de destino não corresponde a este dispositivo.

Importante: essa validação é requisito funcional do MVP. Ela **não deve ser chamada de autenticação criptográfica completa**.

# CLI

O CLI deve ser simples.

Comandos mínimos esperados:

```bash
ducker
ducker id
ducker devices
ducker send <arquivo>
ducker doctor
```

Exemplo:

```bash
ducker send ./foto.png
```

O CLI lista os dispositivos encontrados e permite escolher um.

Progresso deve ser visível.

# Mobile

O Mobile é prioridade porque é o segundo dispositivo disponível para testar o Ducker.

Cenário mínimo:

```text
PC/Notebook
     |
     | arquivo
     v
  Rede local
     |
     v
   Mobile
```

O Mobile deve conseguir:

- configurar nome
- gerar/persistir ID Quac
- descobrir dispositivos
- receber arquivo
- validar destino
- mostrar progresso
- informar sucesso/erro

O Mobile não precisa ter todos os recursos do produto final para validar o MVP.

# Erros

Códigos iniciais:

```text
DEVICE_NOT_FOUND
CONNECTION_FAILED
TRANSFER_FAILED
INVALID_REQUEST
PROTOCOL_VERSION_MISMATCH
DESTINATION_ID_MISMATCH
```

Para `DESTINATION_ID_MISMATCH`:

- rejeitar sessão
- não salvar arquivo
- encerrar transferência
- registrar o erro
- mostrar mensagem amigável

# Protocolo

Prefira protocolos simples, explícitos e fáceis de depurar no MVP.

Não introduza complexidade arquitetural sem necessidade.

Toda mudança no protocolo deve considerar:

- compatibilidade entre CLI e Mobile
- versão do protocolo
- identificação do remetente
- identificação do destino
- metadados
- tamanho do arquivo
- estado da transferência
- confirmação/erro

# Segurança

Não inventar uma arquitetura criptográfica antes de o fluxo básico funcionar.

Entretanto, nunca tratar o ID Quac como segredo.

O ID serve para identificar/validar o destino. Segurança real será aprofundada posteriormente.

Quando segurança avançada entrar no projeto, separar claramente:

- identificação
- autenticação
- autorização
- confidencialidade
- integridade
- proteção contra MITM
- confiança entre dispositivos

# `.quack`, Teams e Segg

Esses conceitos existem no universo do Ducker, mas suas especificações ainda não devem ser inventadas.

Se uma implementação depender deles antes da especificação estar definida:

1. sinalize a dependência;
2. proponha a menor abstração temporária possível;
3. não invente regras definitivas;
4. mantenha a implementação substituível.

# MCP e n8n

MCP e n8n são integrações, não requisitos do Core.

Arquitetura conceitual:

```text
                 Ducker Core
                      |
               API / interfaces
                 /         \
              MCP          n8n
```

O Ducker deve continuar funcionando sem eles.

# Princípios de desenvolvimento

## 1. Simplicidade

Se duas soluções funcionam, prefira a que possui menos partes móveis.

## 2. Modularidade

Não acople UI, networking e regras de negócio sem necessidade.

## 3. Cross-platform

O Core deve ser reutilizável entre CLI e Mobile sempre que tecnicamente razoável.

## 4. Testabilidade

Toda funcionalidade de transferência deve ser testável sem depender exclusivamente da UI.

## 5. Erros explícitos

Falhas devem possuir códigos estáveis e mensagens úteis.

## 6. Não inventar especificação

Quando a memória/documentação não define algo, marque como pendente em vez de transformar uma suposição em regra oficial.

## 7. Documentar decisões

Decisões arquiteturais importantes devem ser registradas em Markdown.

# Ordem de implementação

```text
1. estrutura do workspace
2. identidade
3. ID Quac
4. descoberta
5. conexão
6. handshake
7. validação de destino
8. metadados
9. transferência
10. progresso
11. erros
12. testes
13. CLI
14. Mobile
15. teste real CLI ↔ Mobile
```

O objetivo é chegar rapidamente a:

```text
arquivo.txt
   ↓
ducker send arquivo.txt
   ↓
selecionar "Celular do Fael"
   ↓
rede
   ↓
validação do ID Quac
   ↓
arquivo recebido
```

# Critério de pronto do MVP

O MVP não está pronto porque o código compila.

Ele está pronto quando uma transferência real puder ser executada repetidamente entre CLI e Mobile na mesma rede, com:

- descoberta funcionando
- ID Quac funcionando
- seleção do destinatário funcionando
- transferência funcionando
- validação do destino funcionando
- erro de ID incorreto funcionando
- arquivo recebido corretamente

# Estilo de colaboração

Ao trabalhar no Ducker:

- seja direto;
- preserve decisões já documentadas;
- pergunte somente quando a decisão realmente estiver indefinida;
- não reescreva arquitetura sem motivo;
- não adicione dependências apenas por conveniência;
- priorize código funcional e testável;
- explique mudanças arquiteturais antes de aplicá-las;
- mantenha documentação atualizada quando uma decisão mudar.

Quando o usuário corrigir uma decisão, trate a correção como a nova fonte de verdade para o projeto.
