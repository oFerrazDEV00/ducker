# Testes do Ducker MVP

## Teste principal: CLI -> Mobile

Pré-requisitos:

- computador com Ducker CLI;
- celular com Ducker Mobile;
- ambos conectados à mesma rede;
- ambos configurados com nome e ID Quac.

Procedimento:

1. iniciar o CLI;
2. iniciar o Mobile;
3. confirmar descoberta mútua;
4. conferir nomes e IDs;
5. executar `ducker send <arquivo>`;
6. selecionar o celular;
7. confirmar o recebimento;
8. verificar se o arquivo foi salvo corretamente;
9. conferir o resultado da transferência.

## Teste de ID incorreto

1. iniciar uma sessão;
2. enviar um ID Quac que não corresponde ao receptor;
3. receptor rejeita a transferência;
4. nenhum arquivo é salvo;
5. erro `DESTINATION_ID_MISMATCH` é produzido.

## Teste de dispositivo indisponível

Selecionar um dispositivo que saiu da rede.

Esperado:

```text
DEVICE_NOT_FOUND
```

## Critério de conclusão

O fluxo CLI -> Mobile deve funcionar repetidamente antes de iniciar as camadas mais complexas do projeto.
