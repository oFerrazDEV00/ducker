# Erros do Ducker MVP

## `DESTINATION_ID_MISMATCH`

O ID Quac de destino da transferência não corresponde ao ID Quac do dispositivo que recebeu a conexão.

Comportamento:

- rejeitar transferência;
- não salvar o arquivo;
- encerrar a sessão;
- registrar o erro;
- informar o usuário.

## `DEVICE_NOT_FOUND`

O dispositivo escolhido não foi encontrado ou ficou indisponível na rede.

## `CONNECTION_FAILED`

Não foi possível estabelecer a conexão.

## `TRANSFER_FAILED`

A transferência começou, mas não foi concluída.

## `INVALID_REQUEST`

A mensagem recebida não possui o formato esperado.

## `PROTOCOL_VERSION_MISMATCH`

As versões do protocolo não são compatíveis.
