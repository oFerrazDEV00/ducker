# Protocolo de Transferência Ducker (Baseado em LocalSend v2)

O Ducker adota integralmente o protocolo aberto **LocalSend v2.1**, garantindo interoperabilidade com qualquer aplicativo LocalSend oficial.

A especificação técnica detalhada completa está salva em [`ducker/LOCALSEND_PROTOCOL.md`](LOCALSEND_PROTOCOL.md).

## 1. Descoberta (Discovery)

- **Multicast UDP:** Porta `53317`, endereço `224.0.0.167`.
- **Payload de Anúncio (`MulticastDto`):**
```json
{
  "alias": "Meu Computador",
  "version": "2.1",
  "deviceModel": "Windows",
  "deviceType": "desktop",
  "fingerprint": "6B86D8761853E206...",
  "port": 53317,
  "protocol": "https",
  "download": false,
  "announce": true,
  "quacId": 46100135
}
```
- Ao receber um anúncio, outros dispositivos respondem enviando `POST /api/localsend/v2/register` de volta para a origem ou emitindo anúncio de retorno.

## 2. Negociação de Upload (Prepare Upload)

- Remetente envia metadados via `POST /api/localsend/v2/prepare-upload[?pin=...][&quac=...]` com lista de arquivos.
- Se o destinatário for Ducker e a query `?quac=` for enviada, o receptor valida que `quac == own_quac_id`. Em caso de divergência, retorna `403 DESTINATION_ID_MISMATCH`.
- O receptor responde `200 OK` com um `sessionId` e tokens individuais para cada arquivo, ou `204 No Content` para mensagens de texto exibidas imediatamente.

## 3. Transferência Binária (Upload)

- Para cada arquivo: `POST /api/localsend/v2/upload?sessionId=...&fileId=...&token=...`
- Os bytes são transmitidos em streaming para um arquivo temporário.
- Se fornecido checksum `sha256`, o hash é validado; em caso de divergência, o arquivo temporário é deletado e retorna `422`.
- O arquivo é movido para o diretório de destino com sanitização estrita de caminho contra *path traversal*.
