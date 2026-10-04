# Ducker MVP

## Meta

Fazer uma transferência real de arquivos funcionar entre CLI e Mobile pela rede.

## Critérios de sucesso

- [ ] CLI inicia;
- [ ] Mobile inicia;
- [ ] usuário define o nome do dispositivo;
- [ ] ID Quac é gerado;
- [ ] ID Quac é persistido;
- [ ] dispositivos são descobertos pela rede;
- [ ] nome e ID Quac são exibidos;
- [ ] usuário escolhe um destinatário;
- [ ] arquivo é enviado;
- [ ] arquivo é recebido;
- [ ] ID Quac de destino é validado;
- [ ] ID incorreto é rejeitado;
- [ ] nenhum arquivo é salvo após rejeição;
- [ ] progresso é exibido;
- [ ] erro é exibido de forma clara;
- [ ] teste CLI -> Mobile funciona.

## Ordem de implementação

```text
1. Identidade
2. Descoberta
3. Conexão
4. Handshake
5. Validação do ID Quac
6. Metadados
7. Transferência
8. Progresso
9. Erros
10. Teste CLI <-> Mobile
```

## Fora do escopo

Não bloquear o MVP esperando:

- `.quack`;
- Teams;
- criptografia avançada;
- MCP;
- n8n;
- cloud;
- contas centralizadas.
