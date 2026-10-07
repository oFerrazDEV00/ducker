<p align="center">
  <img src="app/ui/assets/logo.png" alt="Ducker - Be simple, be duck" width="340">
</p>

> **Be simple, be duck.** Transferência de arquivos ponto-a-ponto (P2P) na rede local, rápida, segura e sem servidores externos, implementada 100% em **Rust** sobre o protocolo **LocalSend v2**.

O Ducker transforma cada dispositivo (computador, notebook, celular) em um **nó autônomo (peer)** capaz de descobrir outros dispositivos por multicast UDP e transferir arquivos e mensagens via HTTPS criptografado com certificados TLS autoassinados e SHA-256 fingerprint pinning.

Além disso, o Ducker é **100% interoperável com aplicativos oficiais do LocalSend** em Android, iOS, Windows, macOS e Linux.

---

## 🚀 Como Executar

### 1. Interface Gráfica (Desktop App — Tauri v2)
Basta dar dois cliques em [`ducker-open.bat`](file:///c:/Users/gabri/OneDrive/Documentos/Ducker%20-%20Rafael/ducker-open.bat) ou executar:
```powershell
cargo run -p ducker-app
```
O app abre com janela visual nativa, radar de busca de dispositivos, envio arrastando arquivos, troca instantânea de mensagens e caixa de diálogo para aceitar/recusar recebimentos.

### 2. Linha de Comando (CLI Headless)

A CLI pode ser executada via [`ducker.bat`](file:///c:/Users/gabri/OneDrive/Documentos/Ducker%20-%20Rafael/ducker.bat) ou `cargo run -p ducker-cli -- [comando]`:

```powershell
# Ver identidade deste dispositivo (Apelido, ID Quac, Fingerprint TLS)
ducker id

# Buscar dispositivos na mesma rede Wi-Fi / LAN
ducker devices

# Forçar busca por varredura na sub-rede (se multicast estiver bloqueado)
ducker devices --scan

# Enviar arquivos ou pastas
ducker send .\foto.png --to "Celular"
ducker send .\pasta_documentos --to 192.168.1.50:53317

# Enviar uma mensagem de texto instantânea
ducker text "Olá, estou te enviando pelo Ducker!" --to "Notebook"

# Iniciar o dispositivo em modo receptor (aguardando arquivos)
ducker serve

# Iniciar receptor aceitando arquivos automaticamente sem confirmação
ducker serve --yes

# Rodar receptor em segundo plano (você pode fechar o terminal)
ducker background

# Parar serviço em segundo plano
ducker stop

# Iniciar automaticamente com o Windows ao ligar o PC
ducker autostart enable

# Diagnóstico de rede, portas e permissões
ducker doctor
```

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
