<p align="center">
  <img src="app/ui/assets/logo.png" alt="Ducker - Be simple, be duck" width="340">
</p>

> **Be simple, be duck.** Transferência de arquivos ponto-a-ponto (P2P) na rede local, rápida, segura e sem servidores externos, implementada 100% em **Rust** sobre o protocolo **LocalSend v2**.

O Ducker transforma cada dispositivo (computador, notebook, celular) em um **nó autônomo (peer)** capaz de descobrir outros dispositivos por multicast UDP e transferir arquivos e mensagens via HTTPS criptografado com certificados TLS autoassinados e SHA-256 fingerprint pinning.

Além disso, o Ducker é **100% interoperável com aplicativos oficiais do LocalSend** em Android, iOS, Windows, macOS e Linux.

---

## 📦 Download & Instalação Oficial

O Ducker oferece instaladores profissionais prontos para uso em Windows e Linux, sem necessidade de compilar código ou instalar o Rust:

### 🪟 Windows (Instalador Oficial)
- **Instalador NSIS:** `Ducker_0.2.0_x64-setup.exe` (com assistente gráfico, seleção de componentes e desinstalador seguro).
- **Pacote MSI corporativo:** `Ducker_0.2.0_x64_en-US.msi`.
- **Seleção de Componentes:** Permite instalar o **Ducker Desktop**, o **Ducker CLI** ou ambos.
- **Configuração de PATH:** A CLI é adicionada automaticamente ao PATH do usuário de forma segura e sem duplicatas.
- **Requisitos de Runtime:** Windows 10/11 (ou Windows 7/8 com WebView2 Runtime). O instalador funciona 100% offline se o WebView2 já estiver presente.
- **Desinstalação Segura:** Remove atalhos, binários e PATH, preservando intactos certificados TLS, configurações e arquivos baixados.

### 🐧 Linux (AppImage, DEB e CLI Standalone)
- **AppImage:** Pacote portátil único sem instalação.
- **Pacote DEB:** `ducker_0.2.0_amd64.deb` para distribuições Debian, Ubuntu e derivados.
- **CLI Tarball:** `ducker-cli-0.1.0-linux-x86_64.tar.gz` acompanhado do script `install.sh`.
- **Requisitos de Runtime:** `libwebkit2gtk-4.1-0` (ou 4.0), `libgtk-3-0` e `libayatana-appindicator3-1`.

### 🛠️ Compilando os Instaladores Localmente
- **Windows:** Execute `scripts\build-installer.bat`
- **Linux:** Execute `scripts/build-installer.sh`

---

## 🚀 Como Executar em Desenvolvimento

### 1. Interface Gráfica (Desktop App — Tauri v2)
Basta dar dois cliques em [`ducker-open.bat`](file:///c:/Users/gabri/OneDrive/Documentos/Ducker%20-%20Rafael/ducker-open.bat) ou executar:
```powershell
cargo run -p ducker-app
```
O app abre com janela visual nativa, radar de busca de dispositivos, envio arrastando arquivos, troca instantânea de mensagens e caixa de diálogo para aceitar/recusar recebimentos.

### 2. Interface de Terminal Interativa (TUI) & Linha de Comando (CLI)

O Ducker agora conta com uma **Interface de Terminal (TUI)** de tela cheia, interativa e moderna construída com `ratatui` e `crossterm`, inspirada nas melhores experiências de ferramentas como Codex CLI e Claude Code, com o amarelo `#FFC83D` e o pato mascote central.

#### 🦆 Iniciando a TUI Interativa
Basta executar o executável sem parâmetros em qualquer terminal suportado:
```powershell
ducker
```
A interface de terminal exibe:
- **Cabeçalho:** Marca Ducker amarela, versão `v0.2.0`, status `● ONLINE`, Quac ID e contagem de dispositivos.
- **Radar de Dispositivos (Esquerda):** Lista dinâmica de dispositivos na rede local com navegação via teclado (`↑`/`↓` ou `k`/`j`), tipos de aparelhos (📱 Celular, 💻 Notebook/PC, 🍓 Raspberry Pi) e protocolos (TLS/HTTP).
- **Painel de Transferência & Progresso (Direita Superior):** Indicador em tempo real com barra Gauge amarela, porcentagem, velocidade em MB/s e bytes transferidos.
- **Registro de Atividades & Mensagens (Direita Inferior):** Log em tempo real de eventos, arquivos recebidos e mensagens diretas P2P com carimbo de horário.
- **Aba Duckive:** Pressione `[d]` ou `[2]` para visualizar seu progresso de transferências, nível, itens cosméticos desbloqueados/equipados e a meta épica de 50.000 arquivos (Pato Verde do WhatsApp).
- **Atalhos Rápidos:**
  - `[↑/↓]` ou `[j/k]`: Selecionar dispositivo
  - `[s]`: Enviar arquivos/pastas para o dispositivo selecionado
  - `[t]`: Enviar mensagem de texto rápida (chat P2P)
  - `[r]`: Atualizar radar e forçar varredura na rede local
  - `[d]` ou `[2]`: Alternar para o painel Duckive
  - `[Tab]`: Alternar entre abas (Radar / Duckive / Ajuda)
  - `[?]` ou `[3]`: Manual de comandos e atalhos
  - `[Esc]`: Fechar caixas modais de envio/recebimento
  - `[q]`: Sair da interface e restaurar o terminal com segurança

#### ⚡ Execução Não-Interativa (Scripts e Automação)
Todos os subcomandos originais continuam disponíveis e operam de forma autônoma sem iniciar a interface gráfica ou TUI:

```powershell
# Ver identidade deste dispositivo (Apelido, ID Quac, Fingerprint TLS)
ducker id

# Buscar dispositivos na mesma rede Wi-Fi / LAN
ducker devices

# Forçar busca por varredura na sub-rede (se multicast estiver bloqueado)
ducker devices --scan

# Enviar arquivos ou pastas com contagem precisa de subarquivos
ducker send .\foto.png --to "Celular"
ducker send .\pasta_documentos --to 192.168.1.50:53317

# Enviar uma mensagem de texto instantânea
ducker text "Olá, estou te enviando pelo Ducker!" --to "Notebook"

# Visualizar progresso, cosméticos e conquistas do Duckive no terminal
ducker duckive
ducker duckive --equip "cap_ducker"
ducker duckive --name "Super Duck"

# Iniciar o dispositivo em modo receptor headless
ducker serve
ducker serve --yes # Aceite automático de transferências

# Rodar receptor em segundo plano (você pode fechar o terminal)
ducker background

# Parar serviço em segundo plano
ducker stop

# Iniciar automaticamente com o Windows ao ligar o PC
ducker autostart enable

# Diagnóstico de rede, portas e permissões
ducker doctor

# Abrir a interface gráfica desktop (Ducker App)
ducker open
```

---

## 🎮 Duckive — O Mascote Pessoal do seu Ducker

O **Duckive** é o sistema integrado de gamificação e personalização do Ducker que evolui com seu uso real da rede:
- **Contagem Consistente:** Cada arquivo individual enviado ou recebido com sucesso incrementa seu progresso (inclusive arquivos dentro de pastas recursivas).
- **Prevenção de Progresso Falso:** Transferências com erro, canceladas ou rejeitadas não geram contagem indevida nem desbloqueios falsos.
- **Reconciliação e Preservação:** Conquistas dependentes de contadores (`centurion`, `first_flight`, `whatsapp_green_legend`, `stylist`) são reconciliadas automaticamente sem perder itens personalizados ou progresso preexistente.
- **🟢 Meta Lendária aos 50.000 Arquivos:** Desbloqueia o cosmético lendário **Pato Verde do WhatsApp** (`whatsapp_green_skin`) e o título Nível 10 (*Mestre Esmeralda WhatsApp*).

---

## 🔄 Verificação e Atualizações Automáticas (GUI)

Na aba **Configurações** da interface gráfica, o botão **Verificar atualizações** permite:
- **Consulta Oficial:** Consulta a API do GitHub Releases do repositório oficial (`oFerrazDEV00/ducker`).
- **Comparação SemVer:** Compara a versão atual instalada com a versão estável mais recente.
- **Download Seguro e Automático:** Caso haja uma versão mais recente com instalador compatível (Windows `.exe`/`.msi` ou Linux `.AppImage`/`.deb`), o download HTTPS é iniciado automaticamente com relatório de progresso em tempo real e verificação de integridade criptográfica SHA-256.
- **Cancelamento e Instalação:** Permite cancelar o download a qualquer momento ou acionar a execução do instalador validado diretamente pelo app.

---

## 🏛️ Arquitetura do Workspace

```text
├── core/       ducker-core  — Biblioteca Rust do protocolo LocalSend v2 (HTTPS, Multicast, P2P)
├── cli/        ducker-cli   — Utilitário de linha de comando headless
├── app/        ducker-app   — Aplicativo multiplataforma com Tauri v2 + UI dark/glassmorphism
└── ducker/     Documentação técnica e especificações do protocolo
```

---

## 🔒 Segurança e Identidade

1. **TLS Autoassinado:** Cada nó gera seu próprio certificado x509 na primeira inicialização com chave privada e calcula o fingerprint SHA-256 (hex).
2. **ID Quac:** Mantido como número de 8 dígitos único de cada dispositivo Ducker, enviado como extensão no protocolo para validação opcional de destino.
3. **Sem Nuvem:** 100% dos dados trafegam exclusivamente pela rede local (LAN / Wi-Fi).
