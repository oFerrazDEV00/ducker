# Arquitetura do Ducker P2P (LocalSend v2 em Rust)

## Princípio

Cada dispositivo é um **nó paritário (peer)** no protocolo LocalSend v2. Não existe servidor central nem conceito de "hub PC vs app satélite". Todo dispositivo roda um `Node` que é simultaneamente:
- **Servidor HTTP(S)** ouvindo na porta padrão `53317`
- **Cliente HTTP(S)** para requisições de registro e upload
- **Anunciante e ouvinte UDP Multicast** em `224.0.0.167:53317`

```text
       +---------------------------------------------+
       |             Rede Local (Wi-Fi/LAN)          |
       |  Multicast 224.0.0.167:53317 / HTTPS:53317  |
       +---------------------------------------------+
             ^                      ^              ^
             |                      |              |
      [Ducker App]            [Ducker CLI]   [App LocalSend Oficial]
       (Tauri v2)              (Headless)     (Android / iOS / Mac)
```

## Componentes

### 1. `ducker-core` (Biblioteca Rust)
- **`identity.rs`**: Geração de identidade, ID Quac e certificado TLS autoassinado persistido em `identity.json`.
- **`tls.rs`**: Suporte a HTTPS com rustls (provider `ring`), cálculo de fingerprint SHA-256 e pinning de certificados.
- **`discovery.rs`**: Multicast UDP em todas as interfaces IPv4 locais ativas e fallback via varredura (`scan_subnet`) /24.
- **`server.rs`**: Servidor axum HTTPS com rotas `/api/localsend/v2/{register, info, prepare-upload, upload, cancel}`.
- **`client.rs`**: Cliente reqwest em streaming para envio de arquivos com acompanhamento de progresso byte a byte.
- **`session.rs`**: Gerenciamento de sessões de recebimento, proteção contra *path traversal* e nomes duplicados.
- **`node.rs`**: Fachada unificada (`Node`) e barramento de eventos assíncronos (`NodeEvent`).

### 2. `ducker-cli` (Linha de Comando)
- Utilitário para uso headless, scripts, servidores ou terminais interativos.
- Comandos: `id`, `devices`, `send`, `text`, `serve`, `receive`, `doctor`, `background`, `stop`, `autostart`.

### 3. `ducker-app` (Interface Gráfica — Tauri v2)
- Compila nativamente para Windows, macOS, Linux, Android e iOS.
- Interface em HTML5/CSS/JavaScript vanilla com glassmorphism, radar de dispositivos e envio arrastando arquivos.
- Integração bidirecional em tempo real com eventos do `ducker-core`.
