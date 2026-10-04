# Ducker CLI - MVP

## Primeira execução

```bash
ducker
```

Caso o dispositivo ainda não esteja configurado:

```text
Bem-vindo ao Ducker!

Nome do dispositivo: Notebook do Fael

Seu ID Quac: 84726193
```

## Listar dispositivos

```bash
ducker devices
```

Exemplo:

```text
Dispositivos encontrados:

1. Celular do Fael [12345678]
2. Notebook do Fael [84726193]
```

## Enviar arquivo

```bash
ducker send ./foto.png
```

O CLI apresenta os destinatários descobertos e solicita uma escolha.

## Progresso

```text
Enviando foto.png
[████████████████░░░░] 82%
```

## Identidade

```bash
ducker id
```

## Diagnóstico

```bash
ducker doctor
```

## Comandos mínimos

- `ducker`
- `ducker devices`
- `ducker send <arquivo>`
- `ducker id`
- `ducker doctor`
