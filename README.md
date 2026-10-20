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
- **Instalador NSIS:** `Ducker_1.0.0_x64-setup.exe` (com assistente gráfico, seleção de componentes e desinstalador seguro).
- **Pacote MSI corporativo:** `Ducker_1.0.0_x64_en-US.msi`.
- **Seleção de Componentes:** Permite instalar o **Ducker Desktop**, o **Ducker CLI** ou ambos.
- **Configuração de PATH:** A CLI é adicionada automaticamente ao PATH do usuário de forma segura e sem duplicatas.
- **Requisitos de Runtime:** Windows 10/11 (ou Windows 7/8 com WebView2 Runtime). O instalador funciona 100% offline se o WebView2 já estiver presente.
- **Desinstalação Segura:** Remove atalhos, binários e PATH, preservando intactos certificados TLS, configurações e arquivos baixados.

### 🐧 Linux (AppImage, DEB e CLI Standalone)
- **AppImage:** Pacote portátil único sem instalação.
- **Pacote DEB:** `ducker_1.0.0_amd64.deb` para distribuições Debian, Ubuntu e derivados.
- **CLI Tarball:** `ducker-cli-1.0.0-linux-x86_64.tar.gz` acompanhado do script `install.sh`.
- **Requisitos de Runtime:** `libwebkit2gtk-4.1-0` (ou 4.0), `libgtk-3-0` e `libayatana-appindicator3-1`.

### 📱 Mobile (Android & iOS)
- **Android APK:** `Ducker_1.0.0.apk` com suporte a seleção de arquivos, detecção de rede local e fallback automático de varredura.
- **iOS IPA (Teste / Sideloading):** `Ducker-iOS-unsigned.ipa` compatível com AltStore e Sideloadly (versão de testes não assinada; distribuição na App Store requer assinatura com conta de desenvolvedor Apple).

### 🛠️ Compilando os Instaladores Localmente
- **Windows:** Execute `scripts\build-installer.bat`
- **Linux:** Execute `scripts/build-installer.sh`

---

## 🚀 Como Executar em Desenvolvimento

### 1. Interface Gráfica (Desktop App — Tauri v2)
Basta dar dois cliques em [`ducker-open.bat`](ducker-open.bat) ou executar:
```powershell
cargo run -p ducker-app
```
O app abre com janela visual nativa, radar de busca de dispositivos, envio arrastando arquivos, troca instantânea de mensagens, laboratório de rede **Debugger do Pato**, automação declarativa **.quack** e caixa de diálogo para aceitar/recusar recebimentos.

### 2. Interface de Terminal Interativa (TUI) & Linha de Comando (CLI)

O Ducker conta com uma **Interface de Terminal (TUI)** de tela cheia, interativa e moderna construída com `ratatui` e `crossterm`, com o amarelo `#FFC83D` e o pato mascote central.

#### 🦆 Iniciando a TUI Interativa
Basta executar o executável sem parâmetros em qualquer terminal suportado:
```powershell
ducker
```
A interface de terminal exibe:
- **Cabeçalho:** Marca Ducker amarela, versão `v1.0.0`, status `● ONLINE`, Quac ID e contagem de dispositivos.
- **Radar de Dispositivos (Esquerda):** Lista dinâmica de dispositivos na rede local com navegação via teclado (`↑`/`↓` ou `k`/`j`), tipos de aparelhos (📱 Celular, 💻 Notebook/PC, 🍓 Raspberry Pi) e protocolos (TLS/HTTP).
- **Painel de Transferência & Progresso (Direita Superior):** Indicador em tempo real com barra Gauge amarela, porcentagem, velocidade em MB/s e bytes transferidos.
- **Registro de Atividades & Mensagens (Direita Inferior):** Log em tempo real de eventos, arquivos recebidos e mensagens diretas P2P com carimbo de horário.
- **Aba Duckive:** Pressione `[d]` ou `[2]` para visualizar seu progresso de transferências, nível, itens cosméticos desbloqueados/equipados e a meta épica de 50.000 arquivos (Pato Verde do WhatsApp).
- **Atalhos Rápidos:**
  - `[↑/↓]` ou `[j/k]`: Selecionar dispositivo (Radar) ou cosmético (Duckive)
  - `[/]`: Buscar e filtrar dispositivos por apelido, IP ou modelo
  - `[s]`: Enviar arquivos/pastas para o dispositivo selecionado (com suporte a `[Tab]` para autocompletar caminhos)
  - `[t]`: Enviar mensagem de texto rápida (chat P2P)
  - `[r]`: Atualizar radar e forçar varredura na rede local
  - `[d]` ou `[2]`: Alternar para o painel Duckive
  - `[Enter]` ou `[e]`: Equipar / desequipar cosmético selecionado na aba Duckive
  - `[n]`: Renomear mascote Duckive interativamente
  - `[Tab]`: Alternar entre abas (Radar / Duckive / Ajuda) ou autocompletar caminhos no modal de envio
  - `[?]` ou `[3]`: Manual de comandos e atalhos
  - `[Esc]`: Fechar modais, limpar filtros de busca ou cancelar ações
  - `[q]`: Sair da interface e restaurar o terminal com segurança

#### ⚡ Execução Não-Interativa (Scripts e Automação)
Todos os subcomandos operam de forma autônoma sem iniciar a interface gráfica ou TUI:

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

# Laboratório e diagnóstico de rede (Debugger do Pato)
ducker doctor
ducker doctor --export relatorio-diagnostico.md
ducker debug

# Automação declarativa de envio (.quack)
ducker quack validate backup.quack
ducker quack run backup.quack --dry-run
ducker quack run backup.quack -y

# Abrir a interface gráfica desktop (Ducker App)
ducker open
```

---

## 🩺 Debugger do Pato — Laboratório de Redes

O **Debugger do Pato** transforma o Ducker em uma ferramenta educativa e prática de diagnóstico de rede local, disponível tanto na interface gráfica (aba **Debugger**) quanto na CLI (`ducker doctor` / `ducker debug`):

- **Visão Geral de Interfaces:** Detecta interfaces de rede físicas e virtuais, endereços IPv4 e IPv6, máscaras, prefixos, broadcast e capacidades de loopback/multicast.
- **Auditoria de Portas do Protocolo:** Verifica a disponibilidade das portas locais TCP e UDP 53317 exigidas pelo LocalSend v2, identificando instâncias duplicadas ou conflitos de firewall.
- **Descoberta Multicast e Fallback:** Avalia a integridade do grupo `224.0.0.167:53317` e fornece suporte a varredura controlada de sub-rede.
- **Radar de Dispositivos e Sonda Ativa (Ping Sonda):** Permite testar a conectividade ponta a ponta contra qualquer IP/porta, medindo a latência (RTT em milissegundos), código de resposta HTTP e validação do fingerprint TLS.
- **Diagnóstico Educativo de Anomalias:** Quando um problema é encontrado (ex: isolamento Wi-Fi, portas ocupadas, ausência de rede), apresenta cartões explicativos detalhando:
  1. *O que foi observado*
  2. *O que isso significa*
  3. *Causas prováveis*
  4. *Próximo teste a executar*
  5. *Ação corretiva recomendada*
- **Exportação Segura de Relatórios:** Permite exportar relatórios técnicos em **Markdown** ou **JSON** sanitizados (sem chaves privadas, sem tokens e sem dados de conteúdo).

---

## ⚡ Formato `.quack` — Automação Declarativa de Envio

O Ducker 1.0.0 introduz a especificação oficial do formato **`.quack`** (Esquema v1), permitindo automatizar tarefas de envio de arquivos em lote de forma reprodutível, validável e segura:

### Exemplo de Arquivo `.quack`:
```yaml
schema: "1"
name: "Backup de Documentos e Notas"
description: "Sincronização rápida para celular de trabalho"
target:
  alias: "Celular"
  port: 53317
sources:
  - path: "./documentos"
    include:
      - "*.pdf"
      - "*.docx"
      - "*.xlsx"
    exclude:
      - "*.tmp"
      - "~*"
    recursive: true
options:
  dry_run: false
  conflict: "rename" # "rename" | "skip" | "overwrite"
  max_total_bytes: 1073741824 # 1 GB
  max_file_count: 500
  require_confirmation: true
```

### Segurança e Validação:
- **Sem execução de código:** O parser `.quack` é estritamente declarativo (YAML e JSON) e não executa scripts arbitrários.
- **Prevenção contra Path Traversal:** O uso de `..` para escapar do diretório base é expressamente rejeitado.
- **Simulação Dry-Run:** Permite simular a resolução de arquivos e checar tamanhos antes de transferir qualquer byte.
- **Limites de Segurança:** Impõe limites padrão de tamanho total e quantidade de arquivos para evitar saturação acidental.

---

## 🎮 Duckive — O Mascote Pessoal do seu Ducker

O **Duckive** é o sistema integrado de gamificação e personalização do Ducker que evolui com seu uso real da rede:
- **Contagem Consistente:** Cada arquivo individual enviado ou recebido com sucesso incrementa seu progresso (inclusive arquivos dentro de pastas recursivas).
- **Prevenção de Progresso Falso:** Transferências com erro, canceladas ou rejeitadas não geram contagem indevida nem desbloqueios falsos.
- **Reconciliação e Preservação:** Conquistas dependentes de contadores (`centurion`, `first_flight`, `whatsapp_green_legend`, `stylist`) são reconciliadas automaticamente sem perder itens personalizados ou progresso preexistente.
- **🟢 Meta Lendária aos 50.000 Arquivos:** Desbloqueia o cosmético lendário **Pato Verde do WhatsApp** (`whatsapp_green_skin`) e o título Nível 10 (*Mestre Esmeralda WhatsApp*).

---

## 🔄 Verificação e Atualizações Automáticas (GUI)

Na aba **Configurações** da interface gráfica, o botão **Verificar atualizações** oferece um ciclo completo e seguro de atualização:
- **Consulta Oficial:** Consulta a API do GitHub Releases do repositório oficial (`oFerrazDEV00/ducker`).
- **Comparação SemVer:** Compara a versão atual instalada com a versão estável mais recente.
- **Download Seguro e Atômico:** Quando há uma versão mais recente compatível (Windows `.exe`/`.msi` ou Linux `.AppImage`/`.deb`), o download HTTPS é iniciado automaticamente para um arquivo temporário `.part`, prevenindo corrupção de arquivos em caso de falha ou cancelamento.
- **Validação Criptográfica SHA-256:** O hash do arquivo baixado é calculado em fluxo de dados e comparado com o checksum oficial publicado no release (`SHA256SUMS.txt` ou `<asset>.sha256`).
- **Proteção contra Path Traversal:** Nomes de artefatos recebidos da API são sanitizados cirurgicamente antes de serem gravados no disco.
- **Cancelamento e Execução Segura:** Permite cancelar o download a qualquer momento e acionar a instalação assistida pelo sistema operacional com confirmação do usuário (`msiexec /i` para pacotes MSI no Windows).

---

## 🏛️ Arquitetura do Workspace

```text
├── core/       ducker-core  — Biblioteca Rust do protocolo LocalSend v2 (HTTPS, Multicast, P2P, Diagnostics, .quack)
├── cli/        ducker-cli   — Utilitário de linha de comando headless e TUI interativa (ratatui)
├── app/        ducker-app   — Aplicativo multiplataforma com Tauri v2 + UI dark/yellow (#FFC83D)
└── ducker/     Documentação técnica e especificações do protocolo
```

---

## 🔒 Segurança e Privacidade

1. **TLS Autoassinado:** Cada nó gera seu próprio certificado x509 na primeira inicialização com chave privada e calcula o fingerprint SHA-256 (hex).
2. **ID Quac:** Mantido como número de 8 dígitos único de cada dispositivo Ducker, enviado como extensão no protocolo para validação opcional de destino.
3. **Sem Nuvem e Sem Sniffing:** 100% dos dados trafegam exclusivamente pela rede local (LAN / Wi-Fi). O Ducker não realiza captura universal de pacotes de terceiros nem expõe chaves privadas.
4. **Proteção contra sobrescrita acidental:** Arquivos recebidos recebem nomes únicos incrementais quando já existem no disco, prevenindo perda de dados.

---

## ⚠️ Limitações Conhecidas do v1.0.0

- **Isolamento de Ponto de Acesso (AP Isolation):** Roteadores corporativos ou redes de visitantes com isolamento de clientes ativado impedem a comunicação direta entre dispositivos Wi-Fi. Nesses ambientes, use a opção de varredura por sub-rede ou conecte via cabo/hotspot pessoal.
- **iOS Sideloading:** A compilação móvel de iOS gera um arquivo IPA não assinado destinado a testes locais via AltStore ou Sideloadly. A publicação oficial na Apple App Store requer provisionamento e assinatura paga da Apple Developer.
- **Conexões Móveis:** Em aparelhos Android, certifique-se de que a permissão de economia de energia em segundo plano não suspenda o receptor do Ducker durante transferências volumosas.

---

## 📄 Licença

Este projeto é distribuído sob os termos da licença **MIT**, conforme o arquivo [LICENSE](LICENSE).
