# Identidade e ID Quac

## Primeira execução

Cada instalação do Ducker solicita um nome para o dispositivo.

Exemplo:

```text
Nome do dispositivo: Notebook do Fael
```

Depois disso, o Ducker gera um identificador numérico chamado **ID Quac**.

```text
Nome: Notebook do Fael
ID Quac: 84726193
```

O nome fica associado ao ID Quac daquela instalação.

## Descoberta

Os dispositivos anunciam sua identidade pela rede. O Ducker deve encontrar os IDs Quac disponíveis e apresentar os nomes associados ao usuário.

Exemplo:

```text
Dispositivos encontrados

1. Celular do Fael
   ID Quac: 12345678

2. Notebook do Fael
   ID Quac: 84726193
```

O usuário escolhe o dispositivo pelo nome, enquanto o ID Quac continua visível para conferência.

## Persistência

O ID Quac deve continuar associado à instalação até que exista uma operação explícita para redefini-lo.
