# Protocolo de Transferência do Ducker MVP

## Objetivo

Definir somente o necessário para descobrir dispositivos e transferir um arquivo pela rede.

## Anúncio de descoberta

Cada dispositivo deve anunciar, no mínimo:

- `quac_id`;
- `device_name`;
- endereço necessário para conexão;
- porta de comunicação;
- versão do protocolo.

Exemplo conceitual:

```json
{
  "type": "ducker_discovery",
  "quac_id": 12345678,
  "device_name": "Celular do Fael",
  "address": "192.168.1.20",
  "port": 7878,
  "protocol_version": 1
}
```

## Fluxo de envio

1. remetente descobre os dispositivos;
2. interface mostra nome e ID Quac;
3. usuário escolhe o destinatário;
4. remetente inicia a sessão;
5. remetente informa o ID Quac de destino;
6. receptor valida o ID Quac;
7. se válido, a transferência continua;
8. se inválido, a transferência é rejeitada;
9. arquivo é transmitido;
10. receptor confirma a conclusão.

## Validação do destino

O receptor deve verificar:

```text
received_destination_quac_id == own_quac_id
```

Se não corresponder:

```text
TRANSFER_REJECTED
reason: DESTINATION_ID_MISMATCH
```

Nenhum arquivo deve ser aceito ou salvo como recebido.

Mensagem para o usuário:

> Ocorreu um erro: a transferência foi interceptada porque o ID Quac de destino não corresponde a este dispositivo.

## Observação

Essa validação é requisito funcional do MVP. Ela não deve ser considerada, sozinha, um sistema completo de autenticação ou criptografia. Essas camadas serão aprofundadas posteriormente.
