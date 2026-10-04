# Arquitetura do Ducker MVP

## Princípio

A primeira versão deve ser pequena e funcional. O objetivo é provar uma transferência real entre CLI e Mobile pela rede.

## Componentes

```text
                  Ducker Core
                       |
              +--------+--------+
              |                 |
             CLI              Mobile
              |                 |
              +--------+--------+
                       |
                     Rede
```

O Core será responsável pela lógica compartilhada sempre que possível.

## Core

Responsabilidades mínimas:

- identidade do dispositivo;
- geração e persistência do ID Quac;
- descoberta na rede;
- anúncio do dispositivo;
- descoberta de destinatários;
- conexão;
- handshake;
- validação do destinatário;
- metadados do arquivo;
- transferência dos bytes;
- progresso;
- estados e erros.

## CLI

- configurar nome do dispositivo;
- mostrar ID Quac;
- listar dispositivos descobertos;
- escolher destinatário;
- selecionar arquivo;
- iniciar transferência;
- mostrar progresso;
- mostrar erros.

## Mobile

- configurar nome do dispositivo;
- mostrar ID Quac;
- descobrir dispositivos;
- receber arquivos;
- futuramente também enviar arquivos;
- validar destino;
- mostrar progresso e erros.

## Fora do MVP

- `.quack` detalhado;
- Teams;
- criptografia avançada;
- MCP;
- n8n;
- cloud;
- contas centralizadas;
- permissões complexas.
