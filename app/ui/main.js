// Ducker — LocalSend v2 Protocol Integration & Interactive UI
const hasTauri = typeof window !== 'undefined' && window.__TAURI__ !== undefined;

const invoke = hasTauri ? window.__TAURI__.core.invoke : async (cmd, args) => {
  console.log(`[Mock Invoke] ${cmd}`, args);
  if (cmd === 'get_identity') {
    return {
      alias: 'Fael-PC',
      quac_id: 8492014,
      fingerprint: 'SHA256:7B9A2F45CD8812A9E3',
      port: 53317,
      save_dir: 'C:/Users/Rafael/Downloads'
    };
  }
  if (cmd === 'list_peers' || cmd === 'refresh') {
    return [
      {
        ip: '192.168.0.42',
        port: 53317,
        protocol: 'https',
        info: {
          alias: 'Fael-iPhone',
          deviceModel: 'iPhone 15 Pro',
          deviceType: 'mobile',
          fingerprint: 'IPHONE_7B9A2F45',
          version: '2.1'
        }
      },
      {
        ip: '192.168.0.57',
        port: 53317,
        protocol: 'https',
        info: {
          alias: 'Notebook do Pai',
          deviceModel: 'Dell Inspiron',
          deviceType: 'desktop',
          fingerprint: 'NOTEBOOK_A1B2C3',
          version: '2.1'
        }
      },
      {
        ip: '192.168.0.80',
        port: 53317,
        protocol: 'http',
        info: {
          alias: 'RaspberryPi',
          deviceModel: 'Raspberry Pi 4',
          deviceType: 'headless',
          fingerprint: 'PI_99887766',
          version: '2.1'
        }
      }
    ];
  }
  return null;
};

// State
let currentPeers = [];
let selectedPeerKey = null;
let activeSessionId = null;
let pendingFiles = [];
let myIdentity = null;

// DOM Elements - Navigation & Header
const headerUserName = document.getElementById('headerUserName');
const userInitial = document.getElementById('userInitial');
const networkSubtitle = document.getElementById('networkSubtitle');
const devicesCountBadge = document.getElementById('devicesCountBadge');

// DOM Elements - Main Lists & Cards
const devicesContainer = document.getElementById('devicesContainer');
const detailedDevicesGrid = document.getElementById('detailedDevicesGrid');
const selectRecipient = document.getElementById('selectRecipient');
const filesPreview = document.getElementById('filesPreview');
const btnSelectFiles = document.getElementById('btnSelectFiles');
const nativeFileInput = document.getElementById('nativeFileInput');
const fileDropZone = document.getElementById('fileDropZone');
const btnExecuteSend = document.getElementById('btnExecuteSend');
const recentTransfersList = document.getElementById('recentTransfersList');
const transfersFullHistory = document.getElementById('transfersFullHistory');

// DOM Elements - Settings & Modals
const inputAlias = document.getElementById('inputAlias');
const btnSaveAlias = document.getElementById('btnSaveAlias');
const settingsQuac = document.getElementById('settingsQuac');
const settingsPort = document.getElementById('settingsPort');
const settingsFp = document.getElementById('settingsFp');
const settingsSaveDir = document.getElementById('settingsSaveDir');
const incomingModal = document.getElementById('incomingModal');
const incomingSender = document.getElementById('incomingSender');
const incomingFilesList = document.getElementById('incomingFilesList');
const btnAcceptTransfer = document.getElementById('btnAcceptTransfer');
const btnRejectTransfer = document.getElementById('btnRejectTransfer');
const incomingProgressBox = document.getElementById('incomingProgressBox');
const incomingProgressBar = document.getElementById('incomingProgressBar');
const incomingProgressText = document.getElementById('incomingProgressText');
const modalActionButtons = document.getElementById('modalActionButtons');

// ==================== NAVIGATION HANDLING ====================
function navigateToPage(pageName) {
  // Update desktop sidebar
  document.querySelectorAll('.sidebar-nav .nav-item').forEach(item => {
    item.classList.toggle('active', item.dataset.page === pageName);
  });

  // Update mobile bottom nav
  document.querySelectorAll('.mobile-bottom-nav .mobile-nav-item').forEach(item => {
    item.classList.toggle('active', item.dataset.page === pageName);
  });

  // Update active page content
  document.querySelectorAll('.page-content').forEach(page => {
    page.classList.remove('active');
  });
  const targetPage = document.getElementById(`page-${pageName}`);
  if (targetPage) {
    targetPage.classList.add('active');
  }
}

document.querySelectorAll('.sidebar-nav .nav-item, .mobile-bottom-nav .mobile-nav-item').forEach(btn => {
  btn.addEventListener('click', () => {
    navigateToPage(btn.dataset.page);
  });
});

document.getElementById('linkViewAllDevices')?.addEventListener('click', () => navigateToPage('devices'));
document.getElementById('linkViewAllTransfers')?.addEventListener('click', () => navigateToPage('transfers'));
document.getElementById('btnProfile')?.addEventListener('click', () => navigateToPage('settings'));
document.getElementById('btnMobileSettings')?.addEventListener('click', () => navigateToPage('settings'));

// Mobile FAB button
document.getElementById('mobileFabSend')?.addEventListener('click', (e) => {
  e.preventDefault();
  e.stopPropagation();
  navigateToPage('home');
  const btn = document.getElementById('btnSelectFiles');
  if (btn) {
    btn.scrollIntoView({ behavior: 'smooth', block: 'center' });
    btn.classList.add('pulse-focus');
    setTimeout(() => btn.classList.remove('pulse-focus'), 1200);
  }
  handleFilePicker();
});

// ==================== THEME MANAGEMENT ====================
function initTheme() {
  const savedTheme = localStorage.getItem('ducker_theme') || 'dark';
  applyTheme(savedTheme);

  document.querySelectorAll('.theme-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      const themeVal = btn.dataset.themeVal;
      applyTheme(themeVal);
    });
  });
}

function applyTheme(theme) {
  document.documentElement.setAttribute('data-theme', theme);
  localStorage.setItem('ducker_theme', theme);
  document.querySelectorAll('.theme-btn').forEach(btn => {
    btn.classList.toggle('active', btn.dataset.themeVal === theme);
  });
}

// ==================== IDENTITY MANAGEMENT ====================
async function loadIdentity() {
  try {
    const id = await invoke('get_identity');
    myIdentity = id;

    if (id && id.alias) {
      headerUserName.textContent = id.alias;
      userInitial.textContent = id.alias.charAt(0).toUpperCase();
      inputAlias.value = id.alias;
      settingsQuac.textContent = id.quac_id || '--------';
      settingsPort.textContent = id.port || 53317;
      settingsFp.textContent = id.fingerprint || '--------';
      settingsSaveDir.textContent = id.save_dir || 'Downloads';
    }
  } catch (e) {
    console.error('Falha ao obter identidade:', e);
  }
}

btnSaveAlias?.addEventListener('click', async () => {
  const newAlias = inputAlias.value.trim();
  if (!newAlias) return;
  try {
    await invoke('set_alias', { alias: newAlias });
    await loadIdentity();
    alert('Apelido atualizado com sucesso!');
  } catch (e) {
    alert(`Erro ao atualizar apelido: ${e}`);
  }
});

// ==================== DEVICES RENDERING ====================
function getDeviceIcon(type, model) {
  const t = (type || '').toLowerCase();
  const m = (model || '').toLowerCase();
  if (t === 'mobile' || m.includes('iphone') || m.includes('android')) return '📱';
  if (m.includes('notebook') || m.includes('laptop') || m.includes('macbook')) return '💻';
  if (t === 'headless' || m.includes('raspberry') || m.includes('pi')) return '🍓';
  return '🖥️';
}

function renderDevices(peers) {
  currentPeers = peers || [];
  const count = currentPeers.length;

  networkSubtitle.textContent = `Conectado a ${count} dispositivo${count === 1 ? '' : 's'}`;
  devicesCountBadge.textContent = count;

  // Atualiza opções do dropdown de destinatário
  selectRecipient.innerHTML = '<option value="">Selecione um dispositivo...</option>';

  if (count === 0) {
    devicesContainer.innerHTML = `
      <div class="empty-devices-box">
        <div class="searching-radar">
          <span class="radar-circle rc1"></span>
          <span class="radar-circle rc2"></span>
          <span class="radar-duck">🦆</span>
        </div>
        <p>Buscando dispositivos na rede local...</p>
        <small>Certifique-se de estar no mesmo Wi-Fi com o app aberto.</small>
      </div>
    `;
    detailedDevicesGrid.innerHTML = `<p style="color: var(--text-muted); grid-column: 1/-1;">Nenhum dispositivo encontrado ainda.</p>`;
    selectedPeerKey = null;
    updateSendButtonState();
    return;
  }

  // Verifica se o peer selecionado ainda existe
  const stillExists = selectedPeerKey && currentPeers.some(p => {
    const k = p.key || p.info.fingerprint || `${p.ip}:${p.port}`;
    return k.toLowerCase() === selectedPeerKey.toLowerCase();
  });

  if (!stillExists) {
    if (currentPeers.length > 0) {
      const first = currentPeers[0];
      selectedPeerKey = first.key || first.info.fingerprint || `${first.ip}:${first.port}`;
    } else {
      selectedPeerKey = null;
    }
  }

  devicesContainer.innerHTML = '';
  detailedDevicesGrid.innerHTML = '';

  currentPeers.forEach(peer => {
    const peerKey = peer.key || peer.info.fingerprint || `${peer.ip}:${peer.port}`;
    const isSelected = selectedPeerKey && selectedPeerKey.toLowerCase() === peerKey.toLowerCase();
    const icon = getDeviceIcon(peer.info.deviceType, peer.info.alias || peer.info.deviceModel);

    // Card na Dashboard Principal
    const card = document.createElement('div');
    card.className = `device-item-card ${isSelected ? 'selected' : ''}`;
    card.innerHTML = `
      <div class="device-item-left">
        <div class="device-avatar-box">${icon}</div>
        <div class="device-info-texts">
          <h5>${peer.info.alias || 'Dispositivo'}</h5>
          <p>${peer.ip}</p>
        </div>
      </div>
      <div class="badge-online">
        <span class="dot"></span>
        <span>Online</span>
      </div>
    `;
    card.addEventListener('click', () => {
      selectedPeerKey = peerKey;
      renderDevices(currentPeers);
    });
    devicesContainer.appendChild(card);

    // Card na página detalhada de Dispositivos
    const detCard = document.createElement('div');
    detCard.className = 'device-detailed-card';
    detCard.innerHTML = `
      <div style="display:flex; align-items:center; justify-content:space-between;">
        <div style="display:flex; align-items:center; gap:12px;">
          <span style="font-size:1.6rem;">${icon}</span>
          <div>
            <h4 style="font-weight:700;">${peer.info.alias}</h4>
            <span style="font-size:0.8rem; color:var(--text-dim);">${peer.info.deviceModel || 'Modelo padrão'}</span>
          </div>
        </div>
        <div class="badge-online"><span class="dot"></span> Online</div>
      </div>
      <div style="font-size:0.82rem; color:var(--text-muted); display:flex; flex-direction:column; gap:4px; font-family:var(--font-mono); margin-top:8px;">
        <div>IP: ${peer.ip}:${peer.port} (${peer.protocol.toUpperCase()})</div>
        <div>FP: ${peer.info.fingerprint.slice(0, 16)}...</div>
      </div>
      <button class="btn btn-secondary btn-sm" style="margin-top:8px;">Selecionar para Envio</button>
    `;
    detCard.querySelector('button').addEventListener('click', () => {
      selectedPeerKey = peerKey;
      navigateToPage('home');
      renderDevices(currentPeers);
    });
    detailedDevicesGrid.appendChild(detCard);

    // Opção no Dropdown
    const opt = document.createElement('option');
    opt.value = peerKey;
    opt.textContent = `${peer.info.alias} (${peer.ip})`;
    if (isSelected) opt.selected = true;
    selectRecipient.appendChild(opt);
  });

  updateSendButtonState();
}

selectRecipient.addEventListener('change', (e) => {
  selectedPeerKey = e.target.value;
  renderDevices(currentPeers);
});

// Refresh & Scan Actions
async function refreshPeers() {
  try {
    const peers = await invoke('refresh');
    renderDevices(peers);
  } catch (e) {
    console.error('Erro ao atualizar peers:', e);
  }
}

document.getElementById('btnQuickRefresh')?.addEventListener('click', refreshPeers);
document.getElementById('btnDevicesRefresh')?.addEventListener('click', refreshPeers);

async function scanSubnet() {
  const btn = document.getElementById('linkScanNet');
  const btnDet = document.getElementById('btnDevicesScanSubnet');
  if (btn) btn.textContent = 'Escaneando...';
  if (btnDet) btnDet.textContent = 'Escaneando...';
  try {
    const found = await invoke('scan_network');
    await refreshPeers();
  } catch (e) {
    console.error('Erro na varredura:', e);
  } finally {
    if (btn) btn.textContent = 'Escanear rede';
    if (btnDet) btnDet.textContent = '🌐 Scan Sub-rede /24';
  }
}

document.getElementById('linkScanNet')?.addEventListener('click', scanSubnet);
document.getElementById('btnDevicesScanSubnet')?.addEventListener('click', scanSubnet);

// ==================== FILE SELECTION & TRANSFER ====================
function cleanDisplayName(pathOrName) {
  const raw = pathOrName.split(/[\\/]/).pop() || '';
  const match = raw.match(/^\d+_[a-z0-9]+_(.+)$/i);
  return match ? match[1] : raw;
}

function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

function updateSendButtonState() {
  const hasFiles = pendingFiles.length > 0;
  const hasTarget = Boolean(selectedPeerKey);
  btnExecuteSend.disabled = !(hasFiles && hasTarget);

  if (hasFiles) {
    const names = pendingFiles.map(p => cleanDisplayName(p));
    filesPreview.textContent = `${pendingFiles.length} arquivo(s): ${names.join(', ')}`;
    filesPreview.style.color = 'var(--accent-yellow)';
  } else {
    filesPreview.textContent = 'Nenhum arquivo selecionado';
    filesPreview.style.color = 'var(--text-muted)';
  }
}

let isPickerActive = false;

async function handleFilePicker() {
  if (isPickerActive) return;
  isPickerActive = true;

  try {
    if (hasTauri) {
      // Desktop: rfd nativo
      const selected = await invoke('pick_files');
      if (selected && selected.length > 0) {
        pendingFiles = selected;
        updateSendButtonState();
        return;
      }
    }
  } catch (e) {
    console.warn('Fallback para input HTML5:', e);
  } finally {
    setTimeout(() => { isPickerActive = false; }, 800);
  }

  // Fallback para input HTML5 (indispensável no Android e iOS)
  nativeFileInput.click();
}

// Botão explícito de seleção de arquivos (ÚNICO gatilho de clique para evitar popup acidental)
btnSelectFiles.addEventListener('click', (e) => {
  e.preventDefault();
  e.stopPropagation();
  handleFilePicker();
});

// Drag and drop events (desktop)
['dragenter', 'dragover'].forEach(eventName => {
  fileDropZone.addEventListener(eventName, (e) => {
    e.preventDefault();
    e.stopPropagation();
    fileDropZone.classList.add('dragover');
  });
});

['dragleave'].forEach(eventName => {
  fileDropZone.addEventListener(eventName, (e) => {
    e.preventDefault();
    e.stopPropagation();
    fileDropZone.classList.remove('dragover');
  });
});

fileDropZone.addEventListener('drop', async (e) => {
  e.preventDefault();
  e.stopPropagation();
  fileDropZone.classList.remove('dragover');
  if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length > 0) {
    await processSelectedFiles(Array.from(e.dataTransfer.files));
  }
});

nativeFileInput.addEventListener('change', async (e) => {
  const files = Array.from(e.target.files);
  if (files.length > 0) {
    await processSelectedFiles(files);
  }
  nativeFileInput.value = '';
});

// Processa arquivos selecionados (suporte nativo e staging em chunks no Android/iOS)
async function processSelectedFiles(files) {
  if (!files || files.length === 0) return;

  btnExecuteSend.disabled = true;
  filesPreview.textContent = `Preparando ${files.length} arquivo(s)...`;
  filesPreview.style.color = 'var(--accent-yellow)';

  const stagedPaths = [];
  try {
    for (let i = 0; i < files.length; i++) {
      const file = files[i];

      // Se a plataforma expõe caminho absoluto do sistema operacional (Desktop)
      if (file.path && typeof file.path === 'string' && file.path.length > 0) {
        stagedPaths.push(file.path);
        continue;
      }

      // No Android e iOS WebViews: transferimos os bytes em blocos para o sandbox do app
      const stageId = Date.now() + '_' + Math.random().toString(36).substring(2, 7);
      const CHUNK_SIZE = 1024 * 1024; // 1 MB por bloco
      let offset = 0;
      let isFirst = true;
      let finalPath = '';

      while (offset < file.size) {
        const slice = file.slice(offset, offset + CHUNK_SIZE);
        const buffer = await slice.arrayBuffer();
        const data = Array.from(new Uint8Array(buffer));

        finalPath = await invoke('stage_file_chunk', {
          payload: {
            stage_id: stageId,
            file_name: file.name,
            data,
            is_first: isFirst
          }
        });

        offset += CHUNK_SIZE;
        isFirst = false;

        if (file.size > 2 * 1024 * 1024) {
          const pct = Math.min(100, Math.round((offset / file.size) * 100));
          filesPreview.textContent = `Carregando [${i + 1}/${files.length}] ${file.name}: ${pct}%`;
        }
      }

      if (finalPath) {
        stagedPaths.push(finalPath);
      }
    }

    pendingFiles = stagedPaths;
    updateSendButtonState();
  } catch (err) {
    console.error('Erro ao preparar arquivos:', err);
    alert('Erro ao carregar arquivos selecionados: ' + err);
    pendingFiles = [];
    updateSendButtonState();
  }
}

// Envio de Arquivos
btnExecuteSend.addEventListener('click', async () => {
  if (!selectedPeerKey || pendingFiles.length === 0) return;

  const targetPeer = currentPeers.find(p => {
    const k = p.key || p.info.fingerprint || `${p.ip}:${p.port}`;
    return k.toLowerCase() === selectedPeerKey.toLowerCase();
  });
  const targetName = targetPeer ? targetPeer.info.alias : selectedPeerKey;

  btnExecuteSend.disabled = true;
  btnExecuteSend.textContent = 'Transferindo...';

  try {
    await invoke('send_files', {
      payload: {
        peer_key: selectedPeerKey,
        paths: pendingFiles
      }
    });

    // Registra no histórico com nomes limpos
    pendingFiles.forEach(file => {
      const name = cleanDisplayName(file);
      addTransferHistory({
        name,
        type: getFileCategory(name),
        direction: 'sent',
        peerName: targetName,
        size: 'Concluído',
        date: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
      });
    });

    pendingFiles = [];
    updateSendButtonState();
    // Limpa os arquivos temporários de staging no dispositivo
    invoke('clear_staged_files').catch(console.warn);
    alert('Envio concluído com sucesso! 🦆');
  } catch (e) {
    console.error('Falha no envio:', e);
    alert(`Erro ao transferir arquivos: ${e}`);
  } finally {
    btnExecuteSend.textContent = '🚀 Enviar arquivos';
    updateSendButtonState();
  }
});

// Recado de texto rápido
document.getElementById('btnSendQuickMsg')?.addEventListener('click', async () => {
  const txtInput = document.getElementById('txtQuickMessage');
  const text = txtInput.value.trim();
  if (!text) return;
  if (!selectedPeerKey) {
    alert('Selecione um dispositivo destinatário primeiro!');
    return;
  }
  const btn = document.getElementById('btnSendQuickMsg');
  if (btn) {
    btn.disabled = true;
    btn.textContent = 'Enviando...';
  }
  try {
    await invoke('send_text', {
      payload: { peer_key: selectedPeerKey, text }
    });
    txtInput.value = '';
    alert('Mensagem enviada com sucesso! 🦆');
  } catch (e) {
    alert(`Erro ao enviar mensagem: ${e}`);
  } finally {
    if (btn) {
      btn.disabled = false;
      btn.textContent = 'Enviar Texto';
    }
  }
});

// ==================== TRANSFER HISTORY ====================
function getFileCategory(filename) {
  const ext = (filename.split('.').pop() || '').toLowerCase();
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) return 'zip';
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(ext)) return 'img';
  if (['pdf', 'doc', 'docx', 'txt', 'md'].includes(ext)) return 'doc';
  return 'doc';
}

function getFileIcon(type) {
  if (type === 'zip') return '📦';
  if (type === 'img') return '🖼️';
  return '📄';
}

let transferHistory = [];

function loadHistory() {
  const raw = localStorage.getItem('ducker_transfers_history');
  if (raw) {
    try {
      transferHistory = JSON.parse(raw);
    } catch {
      transferHistory = [];
    }
  }

  // Preenche dados padrão caso esteja vazio para manter a fidelidade ao mockup
  if (transferHistory.length === 0) {
    transferHistory = [
      { name: 'projeto-fard.zip', type: 'zip', direction: 'sent', peerName: 'DESKTOP-FAEL', size: '4.2 MB', date: 'Hoje' },
      { name: 'img_20261004.jpg', type: 'img', direction: 'received', peerName: 'Fael-iPhone', size: '4.2 MB', date: 'Hoje' },
      { name: 'backup.zip', type: 'zip', direction: 'sent', peerName: 'RaspberryPi', size: '45 MB', date: 'Hoje' },
      { name: 'documentacao.md', type: 'doc', direction: 'received', peerName: 'Notebook do Pai', size: '2.1 MB', date: 'Hoje' }
    ];
  }
  renderHistory();
}

function addTransferHistory(item) {
  transferHistory.unshift(item);
  if (transferHistory.length > 50) transferHistory.pop();
  localStorage.setItem('ducker_transfers_history', JSON.stringify(transferHistory));
  renderHistory();
}

function renderHistory() {
  recentTransfersList.innerHTML = '';
  transfersFullHistory.innerHTML = '';

  const recents = transferHistory.slice(0, 4);

  recents.forEach(item => {
    const row = document.createElement('div');
    row.className = 'transfer-item-row';
    const sub = item.direction === 'sent' ? `Enviado para ${item.peerName}` : `Recebido de ${item.peerName}`;

    row.innerHTML = `
      <div class="transfer-item-left">
        <div class="file-type-icon file-type-${item.type}">${getFileIcon(item.type)}</div>
        <div class="transfer-item-info">
          <div class="transfer-item-title">${item.name}</div>
          <div class="transfer-item-sub">${sub} • ${item.size}</div>
        </div>
      </div>
      <div class="transfer-item-status">✓</div>
    `;
    recentTransfersList.appendChild(row);
  });

  // Lista completa
  transferHistory.forEach(item => {
    const row = document.createElement('div');
    row.className = 'transfer-item-row';
    const sub = item.direction === 'sent' ? `Enviado para ${item.peerName}` : `Recebido de ${item.peerName}`;

    row.innerHTML = `
      <div class="transfer-item-left">
        <div class="file-type-icon file-type-${item.type}">${getFileIcon(item.type)}</div>
        <div class="transfer-item-info">
          <div class="transfer-item-title">${item.name}</div>
          <div class="transfer-item-sub">${sub} • ${item.size} • ${item.date}</div>
        </div>
      </div>
      <div class="transfer-item-status">✓</div>
    `;
    transfersFullHistory.appendChild(row);
  });
}

document.getElementById('btnClearHistory')?.addEventListener('click', () => {
  if (confirm('Deseja limpar todo o histórico de transferências?')) {
    transferHistory = [];
    localStorage.removeItem('ducker_transfers_history');
    renderHistory();
  }
});

document.getElementById('btnOpenDownloadsFolder')?.addEventListener('click', () => {
  invoke('open_save_dir').catch(console.error);
});

// ==================== INCOMING TRANSFER MODAL ====================
function showIncomingModal(session_id, sender, files) {
  activeSessionId = session_id;
  incomingSender.textContent = `De: ${sender.alias || 'Dispositivo'} (${sender.deviceModel || 'Rede'})`;
  incomingFilesList.innerHTML = '';

  files.forEach(f => {
    const sizeMb = (f.size / (1024 * 1024)).toFixed(1);
    const div = document.createElement('div');
    div.textContent = `• ${f.fileName} (${sizeMb} MB)`;
    incomingFilesList.appendChild(div);
  });

  incomingProgressBox.style.display = 'none';
  modalActionButtons.style.display = 'flex';
  incomingModal.style.display = 'grid';
}

btnAcceptTransfer?.addEventListener('click', async () => {
  if (!activeSessionId) return;
  modalActionButtons.style.display = 'none';
  incomingProgressBox.style.display = 'block';
  incomingProgressText.textContent = 'Aceito! Recebendo arquivos...';
  await invoke('respond_request', { sessionId: activeSessionId, accept: true });
});

btnRejectTransfer?.addEventListener('click', async () => {
  if (!activeSessionId) return;
  await invoke('respond_request', { sessionId: activeSessionId, accept: false });
  incomingModal.style.display = 'none';
  activeSessionId = null;
});

// ==================== TAURI BACKEND EVENTS ====================
if (hasTauri) {
  window.__TAURI__.event.listen('ducker://event', (e) => {
    const ev = e.payload;
    console.log('[Ducker Core Event]', ev);

    if (ev.type === 'peerDiscovered') {
      invoke('list_peers').then(renderDevices).catch(console.error);
    } else if (ev.type === 'incomingRequest') {
      if (!ev.autoAccepted) {
        showIncomingModal(ev.sessionId, ev.sender, ev.files);
      } else {
        ev.files.forEach(f => {
          addTransferHistory({
            name: f.fileName,
            type: getFileCategory(f.fileName),
            direction: 'received',
            peerName: ev.sender.alias,
            size: `${(f.size / (1024 * 1024)).toFixed(1)} MB`,
            date: 'Agora'
          });
        });
      }
    } else if (ev.type === 'textReceived') {
      alert(`✉️ Mensagem recebida de ${ev.sender.alias}:\n\n"${ev.text}"`);
    } else if (ev.type === 'receiveProgress') {
      const pct = Math.min(100, Math.round((ev.received / ev.total) * 100));
      incomingProgressBar.style.width = `${pct}%`;
      incomingProgressText.textContent = `Recebendo: ${pct}%`;
    } else if (ev.type === 'fileReceived') {
      addTransferHistory({
        name: ev.fileName,
        type: getFileCategory(ev.fileName),
        direction: 'received',
        peerName: 'Dispositivo',
        size: 'Concluído',
        date: 'Agora'
      });
    } else if (ev.type === 'sessionFinished' || ev.type === 'sessionCancelled') {
      incomingModal.style.display = 'none';
      activeSessionId = null;
    }
  });

  window.__TAURI__.event.listen('ducker://send-progress', (e) => {
    const p = e.payload;
    const pct = Math.min(100, Math.round((p.totalSent / p.total) * 100));
    btnExecuteSend.textContent = `Enviando: ${pct}%`;
  });
}

// ==================== INICIALIZAÇÃO ====================
initTheme();
loadIdentity();
loadHistory();
refreshPeers();
setInterval(refreshPeers, 8000);

// Auto-scan rápido após 2.5s se nenhum peer responder via multicast (essencial para redes Wi-Fi móveis)
setTimeout(() => {
  if (currentPeers.length === 0) {
    console.log('[Ducker Discovery] Executando busca ativa na sub-rede...');
    scanSubnet();
  }
}, 2500);
