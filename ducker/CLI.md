# Ducker CLI

Manual de referência dos comandos do Ducker CLI (protocolo LocalSend v2).

## Comandos Disponíveis

### `ducker id`
Exibe o apelido do nó, o ID Quac (8 dígitos) e a impressão digital TLS (fingerprint SHA-256).

### `ducker devices [--scan]`
Descobre dispositivos ativos na mesma rede local ouvindo anúncios UDP multicast.
- `--scan`: Realiza varredura completa na sub-rede /24 caso o multicast esteja bloqueado.

### `ducker send <caminhos...> [--to <destinatário>] [--pin <pin>]`
Envia um ou mais arquivos ou diretórios inteiros.
- Se `--to` for omitido, abre um menu interativo com os dispositivos detectados.
- Aceita como destino: Apelido, ID Quac, prefixo do fingerprint, IP ou IP:Porta.

### `ducker text <mensagem> [--to <destinatário>]`
Envia uma mensagem de texto ou link direto que aparece instantaneamente no dispositivo remoto.

### `ducker serve` (ou `ducker receive`)
Inicia o modo de escuta para receber arquivos.
- `-y, --yes`: Aceita todas as transferências automaticamente.
- `--port <porta>`: Porta TCP personalizada (padrão: 53317).
- `--save-dir <pasta>`: Diretório personalizado para salvar arquivos recebidos.
- `--pin <pin>`: Exige PIN de quem for enviar.

### `ducker background`
Inicia o serviço em segundo plano no Windows (sem janela de terminal) com auto-aceite ativado.

### `ducker stop`
Encerra processos do Ducker rodando em segundo plano.

### `ducker autostart [enable|disable]`
Configura inicialização automática silenciosa ao ligar o computador Windows.

### `ducker doctor`
Executa diagnóstico de interfaces locais IPv4, portas 53317 (TCP e UDP), certificados e permissões.
