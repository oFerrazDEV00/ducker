# HANDOFF — Reescrita do Ducker como protocolo P2P (LocalSend v2) em Rust

> **Para qualquer LLM/dev que continuar este trabalho:** leia este arquivo inteiro primeiro.
> Ele é a fonte da verdade do progresso. Atualize o checklist ao terminar cada item
> e faça commit (`git add -A && git commit -m "..."`) a cada etapa concluída.
> Branch de trabalho: `rewrite/localsend-rust`.

## 1. Objetivo

Antes: o PC (CLI Rust) era um "hub" (`mobile_bridge.rs` + dashboard web) e o celular era um
app Expo/React Native que se conectava por WebSocket ao PC.

Agora: **cada dispositivo é um nó igual (peer)** que fala o protocolo
[LocalSend v2.x](https://github.com/localsend/protocol) — implementado em Rust.
Consequência: o Ducker conversa com apps LocalSend oficiais e vice-versa.

Decisões tomadas com o usuário (não reabrir sem perguntar):

| Tema | Decisão |
|---|---|
| Mobile | **Tauri v2** (Rust + UI HTML/CSS/JS vanilla). App Expo (`mobile/`) foi apagado. |
| Protocolo | **LocalSend v2 real** (multicast `224.0.0.167:53317`, rotas `/api/localsend/v2/*`). |
| Segurança | **HTTPS** com certificado autoassinado (rcgen) + fingerprint = SHA-256 do cert (DER, hex). |
| ID Quac | Mantido como campo extra opcional `quacId` no DeviceInfo + validação extra opcional via query `?quac=` no `prepare-upload`. |
| Código antigo | Apagado (está no histórico git, commit `99caf70`). |

Spec completa salva localmente em [`ducker/LOCALSEND_PROTOCOL.md`](ducker/LOCALSEND_PROTOCOL.md).

## 2. Arquitetura alvo

```
Cargo workspace (raiz)
├── core/   ducker-core  — biblioteca do protocolo (sem UI). Usada pela CLI e pelo app.
│   └── src/
│       ├── lib.rs          re-exports
│       ├── error.rs        DuckerError
│       ├── model.rs        DTOs do LocalSend (DeviceInfo, FileDto, PrepareUpload*, ...)
│       ├── identity.rs     Identity { alias, quac_id, cert_pem, key_pem, fingerprint }
│       ├── tls.rs          geração de cert, ServerConfig rustls, verifier do cliente (pinning)
│       ├── discovery.rs    multicast UDP (announce/listen) + scan HTTP legado da sub-rede
│       ├── server.rs       servidor axum HTTPS: register/info/prepare-upload/upload/cancel
│       ├── client.rs       cliente reqwest: register, prepare-upload, upload, cancel
│       ├── session.rs      estado das sessões de recebimento (tokens, arquivos, progresso)
│       └── node.rs         `Node` = fachada: start(), peers(), send_files(), respond(), eventos
├── cli/    ducker (bin) — peer headless: `ducker serve`, `ducker devices`, `ducker send`, ...
└── app/    Tauri v2 (desktop + Android + iOS)
    ├── src-tauri/          crate `ducker-app`, comandos Tauri que chamam ducker-core::Node
    └── ui/                 index.html, styles.css, main.js (usa window.__TAURI__)
```

### API pública do core (contrato entre core ↔ CLI/app)

```rust
let identity = Identity::load_or_create(&config_dir, "Meu PC")?;   // cria cert na 1ª vez
let config   = NodeConfig::new(identity, save_dir);                  // port 53317, https, etc
let (node, mut events) = Node::start(config).await?;                 // sobe server + discovery

node.peers().await                         -> Vec<Peer>
node.announce().await                      // força novo anúncio multicast
node.scan_subnet().await                   // fallback HTTP legado
node.send_files(&peer, vec![path], cb).await -> Result<()>  // cb(SendProgress)
node.send_text(&peer, "olá").await
node.respond(session_id, accept: bool).await // responde IncomingRequest
node.cancel_receive(session_id).await

events: tokio::sync::broadcast::Receiver<NodeEvent>
  NodeEvent::PeerDiscovered(Peer)
  NodeEvent::IncomingRequest { session_id, sender: DeviceInfo, files: Vec<FileDto> }
  NodeEvent::TextReceived { sender, text }
  NodeEvent::ReceiveProgress { session_id, file_id, received, total }
  NodeEvent::FileReceived { session_id, file_id, path }
  NodeEvent::SessionFinished { session_id }
  NodeEvent::SessionCancelled { session_id }
```

Se `NodeConfig.auto_accept == true`, pedidos são aceitos sem emitir espera (útil p/ CLI `--yes`).
Caso contrário o servidor espera até `RESPOND_TIMEOUT` (60s) por `node.respond(...)`.

## 3. Checklist de progresso

Marque `[x]` ao concluir. Itens em ordem de dependência.

### Fase 0 — Preparação
- [x] Branch `rewrite/localsend-rust` criada
- [x] Este HANDOFF.md
- [x] Salvar spec em `ducker/LOCALSEND_PROTOCOL.md`

### Fase 1 — Core (`core/`)
- [x] Remover módulos antigos: `mobile_bridge.rs`, `dashboard.html`, `transfer.rs`, `protocol.rs`, `dialog.rs`, testes antigos
- [x] `Cargo.toml` com deps novas (rustls com provider **ring** — NÃO usar aws-lc-rs, quebra build Windows/Android)
- [x] `model.rs` (DTOs camelCase, campos opcionais, `deviceType` desconhecido → desktop)
- [x] `identity.rs` + `tls.rs` (cert autoassinado, fingerprint, persistência JSON)
- [x] `session.rs`
- [x] `server.rs` (HTTPS + HTTP, rotas v2; v1 opcional não implementado)
- [x] `client.rs` (aceita cert autoassinado; checa fingerprint quando conhecido)
- [x] `discovery.rs` (multicast join em todas interfaces IPv4, announce, responder via /register, fallback UDP)
- [x] `node.rs` (fachada + eventos)
- [x] Testes: envio ponta-a-ponta entre 2 nós em portas diferentes em 127.0.0.1; rejeição; quac mismatch; sha256 mismatch
- [x] `cargo test -p ducker-core` verde

### Fase 2 — CLI (`cli/`)
- [x] Reescrever comandos: `id`, `serve` (alias `receive`), `devices`, `send <arquivos...> --to <alias|fingerprint|quac|ip[:porta]>`, `text "msg" --to`, `doctor`, `background`, `stop`, `autostart`
- [x] Remover `mobile.rs`, `open.rs`
- [x] Prompt de aceite via `dialoguer` (ou `--yes`)
- [x] `cargo build -p ducker-cli` verde

### Fase 3 — App Tauri (`app/`)
- [x] Apagar `mobile/` (Expo)
- [x] `app/src-tauri` (Cargo.toml, build.rs, tauri.conf.json, capabilities, icons)
- [x] Comandos Tauri: `get_identity`, `set_alias`, `list_peers`, `refresh`, `send_files`, `send_text`, `respond_request`, `open_save_dir`
- [x] Eventos Tauri emitidos a partir de `NodeEvent` (`ducker://event`)
- [x] UI premium (dark, glassmorphism) em `app/ui/`
- [x] `cargo check -p ducker-app` verde no Windows
- [x] `cargo build -p ducker-app` gerando `ducker-app.exe` nativo com seletor nativo de arquivos via `rfd`
- [ ] Android: `tauri android init` + MulticastLock (requer Java/Android SDK na máquina; fallback `scan_subnet()` já implementado no core)

### Fase 4 — Docs e limpeza
- [x] Atualizar `ducker/ARCHITECTURE.md`, `ducker/TRANSFER_PROTOCOL.md`, `ducker/CLI.md`, `README.md`
- [x] Atualizar `.bat` (ducker-open.bat → abre app visual `ducker-app.exe`)
- [ ] Teste manual com app LocalSend oficial no smartphone

## 4. Pendências / armadilhas conhecidas

- **Android multicast:** precisa `WifiManager.MulticastLock` (Java/Kotlin). Requer instalação do Java (JDK) e Android Studio/SDK para executar `npx @tauri-apps/cli android init`. O core já possui `scan_subnet()` (/24 HTTP scan) como fallback funcional caso o lock não esteja presente.
- **iOS:** precisa entitlement `com.apple.developer.networking.multicast` (pedido à Apple). Até lá, usar scan.
- **Diretório de config no mobile:** `dirs::home_dir()` não serve; o app passa `app.path().app_data_dir()`
  para `Identity::load_or_create`. Save dir no Android: `app.path().download_dir()` ou documento do app.
- **Firewall Windows:** porta TCP+UDP 53317 precisa estar liberada (o `ducker doctor` valida e avisa).
- **rustls provider:** chamar `ducker_core::tls::install_crypto_provider()` uma vez no início (feito em `Node::start`).
- **Fingerprint:** LocalSend usa SHA-256 do certificado em hex. Comparar sempre *case-insensitive*.
- Rotas v1 (`/api/localsend/v1/*`) não implementadas — só v2.

## 5. Como validar rapidamente

```powershell
cargo test -p ducker-core
cargo run -p ducker-cli -- doctor
cargo run -p ducker-cli -- id
cargo run -p ducker-cli -- serve --yes          # terminal 1
cargo run -p ducker-cli -- devices              # terminal 2
cargo run -p ducker-cli -- send .\README.md --to <alias>
.\ducker-open.bat                               # abre o app visual desktop Tauri
```

## 6. Log de sessões

- 2026-10-07 — Sessão 1 (Opus): plano criado, decisões registradas, início da Fase 1.
- 2026-10-07 — Sessão 2 (Gemini):
  * Corrigido erro de lifetime em `tls.rs` e teste cross-platform em `session.rs`.
  * Fase 1: 100% dos testes unitários e de integração e2e passando (15 testes verdes no `ducker-core`).
  * Fase 2: CLI totalmente reescrita com `id`, `devices`, `send`, `text`, `serve`, `doctor`, `background`, `stop`.
  * Fase 3: App Tauri v2 criado (`app/src-tauri` e `app/ui/`) com UI moderna glassmorphism, radar, envio de arquivos com diálogo nativo do sistema via `rfd`, mensagens instantâneas e notificações, compilado com sucesso (`ducker-app.exe` e `ducker.exe` em `target/debug/`).
  * Fase 4: `.bat` atualizados, `README.md`, `ARCHITECTURE.md`, `CLI.md`, `TRANSFER_PROTOCOL.md` atualizados.
  * Commits efetuados na branch `rewrite/localsend-rust`:
    - `d063d06` feat(app): implement Tauri v2 desktop app, update docs and launchers for LocalSend v2
    - `dd99f44` feat(app): integrate native file picker dialog via rfd for desktop UI
    - `8cd515b` fix(stability): fix peer flickering with TTL retention, broadcast discovery, and robust peer lookup for transfers
