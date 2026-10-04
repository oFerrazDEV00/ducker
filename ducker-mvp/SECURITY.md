# Segurança do Ducker MVP

## Escopo

O MVP não tenta resolver toda a segurança do ecossistema. Ele precisa apenas implementar a validação funcional do destinatário.

## Validação obrigatória

O receptor compara o ID recebido com o próprio ID:

```text
received_destination_quac_id == own_quac_id
```

Se forem diferentes:

```text
reject()
```

O arquivo não deve ser salvo.

## Importante

ID Quac é, nesta fase, um identificador de destino. Ele não deve ser tratado como prova criptográfica de identidade.

Criptografia e autenticação robustas serão projetadas depois que a transferência básica estiver funcionando.
