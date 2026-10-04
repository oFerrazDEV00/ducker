# Ducker MVP

> Be simple, be duck. 🦆

Ducker é um projeto open source para transferência de arquivos pela rede, com foco local-first.

## Objetivo desta fase

Fazer a transferência de arquivos funcionar de ponta a ponta antes de aprofundar recursos como `.quack`, Teams e criptografia avançada.

O primeiro MVP terá:

- CLI
- Mobile
- descoberta de dispositivos pela rede
- identificação por ID Quac
- nome associado ao ID Quac
- seleção do destinatário
- envio e recebimento de arquivos
- validação do ID Quac de destino

## Fluxo básico

```text
Dispositivo A
    |
    | descoberta
    v
Rede
    |
    +--> ID Quac 12345678 - Celular do Fael
    +--> ID Quac 87654321 - Notebook do Fael
                    |
                    v
             escolher destinatário
                    |
                    v
              enviar arquivo
```

> Neste documento, "Docker" mencionado durante o planejamento deve ser entendido como **Ducker**.
