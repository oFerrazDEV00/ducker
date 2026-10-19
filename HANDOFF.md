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
    - `2ff61c0` fix(tls): support LocalSend mTLS by presenting client certificate during TLS handshake
- 2026-10-07 — Sessão 3 (Gemini):
  * **Redesign completo da UI (Desktop & Mobile):**
    - Layout desktop 1:1 com os mockups Dark/Light/Yellow fornecidos pelo usuário.
    - Sidebar de navegação esquerda ("Início", "Dispositivos", "Transferências", "Configurações") com o mascote Ducker pixel art e slogan *"Be simple, be duck."*.
    - Header com boas-vindas ao usuário, badge "Online" pulsante e avatar.
    - Card de topo com indicador de "Rede local ativa" e contagem dinâmica de peers.
    - Cards de dispositivos modernos com ícones dedicados (mobile, desktop, laptop, raspberry pi) e status verde.
    - Painel direito de transferência com drag & drop, botão amarelo "Selecionar arquivos" e lista de transferências recentes persistida no `localStorage`.
    - Suporte a temas alternáveis (Dark, Light, Yellow) em Configurações.
    - Layout responsivo mobile com bottom navigation bar ("Início", "Dispositivos", "Transferências", "Mais") e botão de ação amarelo flutuante `+ Enviar arquivos`.
  * **Estrutura de Builds Mobile (APK e IPA):**
    - Ajustadas dependências do Tauri no `Cargo.toml` (`rfd` isolado condicionalmente apenas em plataformas desktop).
    - Criado workflow de CI/CD do GitHub Actions (`.github/workflows/build-mobile.yml`):
      * Job Android: compilação automática dos APKs (arm64, armv7, x86_64) via Ubuntu runner com NDK e upload como artefatos de download.
      * Job iOS: compilação automática do IPA unsigned compatível com AltStore/Sideloadly via runner macOS.
    - Criados scripts auxiliares locais em `scripts/` (`setup-mobile-env.bat` e `build-apk-local.bat`).
- 2026-10-19 — Sessão 4 (Gemini):
  * **Sistema Oficial de Instalação e Distribuição (Windows e Linux):**
    - Criado `LICENSE` oficial com licença MIT atribuída a `oFerrazDEV00`.
    - Identidade visual aplicada ao instalador NSIS: gerados bitmaps de cabeçalho (`app/src-tauri/nsis/header.bmp`, 150x57) e barra lateral (`app/src-tauri/nsis/sidebar.bmp`, 164x314) com a paleta amarela `#FFC83D`, mascote pato de borracha e slogan *"Be Simple, Be Duck."*.
    - `tauri.conf.json` configurado com `bundle.windows.nsis` (suporte a `currentUser` e `perMachine`, seletor de idiomas, hooks personalizados e caminhos de imagens) e `bundle.linux` (dependências de sistema `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1` para pacote `.deb` e AppImage).
    - **Seleção de Componentes e PATH:** implementado em `app/src-tauri/nsis/hooks.nsh` com prompt interativo e empacotamento de recursos (`app/src-tauri/resources/ducker.exe`). Usuário escolhe entre Desktop + CLI ou apenas Desktop.
    - **Gerenciamento Seguro de PATH:** adição idempotente ao `HKCU\Environment\Path` sem duplicatas e remoção cirúrgica na desinstalação via broadcast de `WM_SETTINGCHANGE`.
    - **Ciclo de Vida e Desinstalação:** preservação de arquivos de usuário (`%APPDATA%\dev.oferraz.ducker\`), downloads e chaves/certificados TLS.
    - **Scripts de Build e Automação:**
      * Windows: `scripts/build-installer.bat` (compila CLI, copia recursos e gera MSI + NSIS via Tauri).
      * Linux: `scripts/build-installer.sh` (compila CLI, gera AppImage, `.deb` e tarball com `install.sh`).
    - **CI/CD Desktop:** `.github/workflows/build-desktop.yml` para compilação automatizada no Windows (NSIS, MSI, CLI) e Linux Ubuntu 22.04 LTS (AppImage, DEB, CLI tarball).
    - **Validação:** todos os 22 testes da workspace aprovados (`ducker-core` e `e2e`), CLI release validada e artefatos de instalador gerados com sucesso (`Ducker_0.2.0_x64-setup.exe` de 6.27 MiB e `Ducker_0.2.0_x64_en-US.msi` de 8.86 MiB).
- 2026-10-19 — Sessão 5 (Gemini):
  * **Interface de Terminal (TUI) Interativa e Moderna (`cli/src/tui/`):**
    - Implementada experiência de terminal de tela inteira inspirada em Codex CLI e Claude Code usando `ratatui 0.29` e `crossterm 0.28`.
    - Identidade visual refinada: amarelo `#FFC83D`, tons escuros para blocos, pato de borracha Unicode/ASCII como mascote central e badges de status.
    - Executar `ducker` sem subcomandos em qualquer TTY abre a TUI automaticamente.
    - Suporte a detecção de terminal não interativo (`std::io::stdout().is_terminal()`), fazendo fallback para o receptor CLI em pipes, scripts e automações.
    - Layout adaptável com proteção contra redimensionamento em terminais pequenos (< 65 colunas ou < 16 linhas).
    - Panic hook configurado para restaurar raw mode, alternate screen e cursor em qualquer situação de encerramento.
    - Abas interativas: `[1] Radar & Transferências`, `[2] Duckive Companion`, `[3] Ajuda & Comandos`.
    - Modais interativos com captura de teclado para aceitar/recusar transferências recebidas (`[y]/[n]`), envio de mensagens de texto (`[t]`) e envio de arquivos (`[s]`).
    - Navegação rápida via teclado (`↑`/`↓` ou `k`/`j`), atalhos `[r]` (re-escanear), `[d]` (Duckive) e `[q]` (sair).
  * **Melhorias e Correções no Duckive:**
    - Corrigida a contagem de arquivos para percorrer recursivamente árvores de diretórios via `ducker_core::count_files_and_bytes`.
    - Evitado progresso falso ou contagem duplicada em transferências vazias (`files == 0 && bytes == 0 && !is_text`).
    - Implementada reconciliação segura do catálogo (`reconcile_catalog`) sem perder cosméticos equipados, cosméticos customizados de versões anteriores ou conquistas já desbloqueadas.
    - Confirmada e implementada a conquista épica `whatsapp_green_legend` aos 50.000 arquivos enviados, liberando o cosmético lendário `whatsapp_green_skin` (Pato Verde do WhatsApp) e nível 10 ("Mestre Esmeralda WhatsApp").
  * **Verificador e Atualizador Automático de Versões (GUI Tauri):**
    - Criado módulo `app/src-tauri/src/updater.rs` com comparador semver (`is_newer_version`), consulta à API oficial do GitHub Releases (`oFerrazDEV00/ducker`), seleção de asset da plataforma (.exe/.msi no Windows, AppImage/.deb no Linux).
    - Download streaming HTTPS com cálculo simultâneo de hash SHA-256, cancelamento atômico via `AtomicBool` e emissão de progresso (`ducker://update-download-progress`).
    - Execução do instalador validado no sistema operacional (`updater::run_installer`).
    - Card "Atualizações do Ducker" na aba Configurações com botão visível "Verificar atualizações", barra de progresso em tempo real, botão de cancelar e botão de instalar/reiniciar.
  * **Validação Completa da Workspace:**
    - `cargo fmt --check`: 100% formatado e em conformidade.
    - `cargo check --workspace`: 0 erros, 0 avisos.
    - `cargo test --workspace`: 28 testes aprovados (1 em `app`, 3 em `cli`, 16 em `core`, 8 em `e2e`).
    - Subcomandos legados (`id`, `devices`, `duckive`, `doctor`) testados e operacionais.


