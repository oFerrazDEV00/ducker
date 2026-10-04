# Ducker Mobile - MVP

## Objetivo

O Mobile é prioridade porque permite testar o Ducker sem precisar de um segundo computador.

Cenário principal:

```text
PC/Notebook com CLI
        |
        | arquivo
        v
       Rede
        |
        v
      Celular
```

## Primeira execução

O aplicativo solicita um nome:

```text
Nome do dispositivo

[ Celular do Fael ]
```

Depois mostra o ID Quac gerado:

```text
Seu ID Quac

12345678
```

## Tela principal

Deve mostrar, no mínimo:

- nome do dispositivo;
- ID Quac;
- dispositivos encontrados;
- transferências recebidas;
- estado da transferência.

## Recebimento

Quando uma transferência chegar:

```text
Recebendo arquivo

foto.png
De: Notebook do Fael
Quac: 84726193
```

O receptor valida o ID Quac antes de aceitar o arquivo.

## ID incorreto

Se o ID de destino não for o ID deste dispositivo:

```text
Transferência rejeitada

Ocorreu um erro: a transferência foi interceptada porque o ID Quac de destino não corresponde a este dispositivo.
```

O arquivo não deve ser salvo.
