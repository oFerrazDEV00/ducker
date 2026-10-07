// Ducker Frontend Logic — LocalSend v2 Protocol Integration

const hasTauri = typeof window !== 'undefined' && window.__TAURI__ !== undefined;
const invoke = hasTauri ? window.__TAURI__.core.invoke : async (cmd, args) => {
  console.log(`[Mock Invoke] ${cmd}`, args);
  if (cmd === 'get_identity') return { alias: 'Meu PC (Mock)', quac_id: 12345678, fingerprint: 'ABCD1234', port: 53317, save_dir: 'C:/Downloads' };
  if (cmd === 'list_peers' || cmd === 'refresh') return [];
  return null;
};

let currentPeers = [];
let selectedPeerKey = null;
let activeSessionId = null;
let pendingFiles = [];

// DOM Elements
const myAliasEl = document.getElementById('myAlias');
const myQuacEl = document.getElementById('myQuac');
const myFpEl = document.getElementById('myFp');
const devicesListEl = document.getElementById('devicesList');
const targetSelectEl = document.getElementById('targetSelect');
const btnRefresh = document.getElementById('btnRefresh');
const btnScanNet = document.getElementById('btnScanNet');
const btnOpenDownloads = document.getElementById('btnOpenDownloads');
const btnEditAlias = document.getElementById('btnEditAlias');
const textPayloadEl = document.getElementById('textPayload');
const btnSendText = document.getElementById('btnSendText');
const dropZone = document.getElementById('dropZone');
const fileInput = document.getElementById('fileInput');
const btnChooseFiles = document.getElementById('btnChooseFiles');
const btnSendFiles = document.getElementById('btnSendFiles');
const selectedFilesSummary = document.getElementById('selectedFilesSummary');
const activityList = document.getElementById('activityList');

// Modal Elements
const incomingModal = document.getElementById('incomingModal');
const incomingSender = document.getElementById('incomingSender');
const incomingFilesList = document.getElementById('incomingFilesList');
const btnAcceptTransfer = document.getElementById('btnAcceptTransfer');
const btnRejectTransfer = document.getElementById('btnRejectTransfer');
const incomingProgressBox = document.getElementById('incomingProgressBox');
const incomingProgressBar = document.getElementById('incomingProgressBar');
const incomingProgressText = document.getElementById('incomingProgressText');
const modalActionButtons = document.getElementById('modalActionButtons');

// Tabs
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
    btn.classList.add('active');
    document.getElementById(btn.dataset.tab).classList.add('active');
  });
});

async function loadIdentity() {
  try {
    const id = await invoke('get_identity');
    myAliasEl.textContent = id.alias;
    myQuacEl.textContent = id.quac_id;
    myFpEl.textContent = `fp: ${id.fingerprint.slice(0, 8)}...`;
    myFpEl.title = `Fingerprint completo: ${id.fingerprint}`;
  } catch (e) {
    console.error('Falha ao obter identidade:', e);
  }
}

function renderDevices(peers) {
  currentPeers = peers;
  targetSelectEl.innerHTML = '<option value="">Selecione um dispositivo...</option>';

  if (peers.length === 0) {
    devicesListEl.innerHTML = `
      <div class="empty-state">
        <div class="radar-anim">
          <div class="radar-ring r1"></div>
          <div class="radar-ring r2"></div>
          <div class="radar-ring r3"></div>
          <div class="radar-core">🦆</div>
        </div>
        <p>Procurando dispositivos na rede local...</p>
        <small>Certifique-se de estar no mesmo Wi-Fi com o app aberto.</small>
      </div>
    `;
    return;
  }

  devicesListEl.innerHTML = '';
  peers.forEach(peer => {
    const card = document.createElement('div');
    card.className = `device-card ${selectedPeerKey === peer.info.fingerprint ? 'selected' : ''}`;

    const isDucker = peer.info.quacId !== null && peer.info.quacId !== undefined;
    const typeIcon = peer.info.deviceType === 'mobile' ? '📱' : '💻';

    card.innerHTML = `
      <div class="device-left">
        <div class="device-icon-box">${typeIcon}</div>
        <div class="device-meta">
          <div class="device-title">
            ${peer.info.alias}
            ${isDucker ? `<span class="badge-ducker">Quac: ${peer.info.quacId}</span>` : ''}
          </div>
          <div class="device-sub">${peer.protocol}://${peer.ip}:${peer.port}</div>
        </div>
      </div>
      <button class="btn btn-secondary btn-sm btn-select-peer">Selecionar</button>
    `;

    card.addEventListener('click', () => selectPeer(peer));
    devicesListEl.appendChild(card);

    // Option no select
    const opt = document.createElement('option');
    opt.value = peer.info.fingerprint || `${peer.ip}:${peer.port}`;
    opt.textContent = `${peer.info.alias} (${peer.ip})`;
    if (selectedPeerKey === opt.value) opt.selected = true;
    targetSelectEl.appendChild(opt);
  });
}

function selectPeer(peer) {
  selectedPeerKey = peer.info.fingerprint || `${peer.ip}:${peer.port}`;
  targetSelectEl.value = selectedPeerKey;
  document.querySelectorAll('.device-card').forEach(c => c.classList.remove('selected'));
  renderDevices(currentPeers);
}

targetSelectEl.addEventListener('change', (e) => {
  selectedPeerKey = e.target.value;
  renderDevices(currentPeers);
});

async function refreshPeers() {
  btnRefresh.disabled = true;
  btnRefresh.querySelector('.refresh-icon').style.animation = 'spin 1s linear infinite';
  try {
    const peers = await invoke('refresh');
    renderDevices(peers);
  } catch (e) {
    console.error('Erro ao atualizar peers:', e);
  } finally {
    btnRefresh.disabled = false;
    btnRefresh.querySelector('.refresh-icon').style.animation = '';
  }
}

btnRefresh.addEventListener('click', refreshPeers);

btnScanNet.addEventListener('click', async () => {
  btnScanNet.disabled = true;
  btnScanNet.textContent = 'Buscando...';
  try {
    const found = await invoke('scan_network');
    addActivity(`Varredura concluída: ${found} dispositivo(s) encontrado(s)`);
    await refreshPeers();
  } catch (e) {
    console.error('Erro no scan:', e);
  } finally {
    btnScanNet.disabled = false;
    btnScanNet.textContent = '🌐 Scan Sub-rede';
  }
});

btnOpenDownloads.addEventListener('click', () => {
  invoke('open_save_dir').catch(e => console.error(e));
});

btnEditAlias.addEventListener('click', async () => {
  const current = myAliasEl.textContent;
  const next = prompt('Novo apelido para este dispositivo:', current);
  if (next && next.trim() && next !== current) {
    try {
      await invoke('set_alias', { alias: next.trim() });
      myAliasEl.textContent = next.trim();
    } catch (e) {
      alert(`Erro: ${e}`);
    }
  }
});

// Envio de Texto
btnSendText.addEventListener('click', async () => {
  if (!selectedPeerKey) {
    alert('Por favor, selecione um dispositivo de destino na lista.');
    return;
  }
  const text = textPayloadEl.value.trim();
  if (!text) return;

  btnSendText.disabled = true;
  btnSendText.textContent = 'Enviando...';
  try {
    await invoke('send_text', {
      payload: { peer_key: selectedPeerKey, text }
    });
    addActivity(`Mensagem enviada com sucesso!`);
    textPayloadEl.value = '';
  } catch (e) {
    alert(`Falha no envio: ${e}`);
  } finally {
    btnSendText.disabled = false;
    btnSendText.textContent = 'Enviar Mensagem';
  }
});

// Seleção de Arquivos
btnChooseFiles.addEventListener('click', () => fileInput.click());

fileInput.addEventListener('change', (e) => {
  const files = Array.from(e.target.files);
  pendingFiles = files;
  if (files.length > 0) {
    selectedFilesSummary.textContent = `${files.length} arquivo(s) selecionado(s): ${files.map(f => f.name).join(', ')}`;
    btnSendFiles.disabled = false;
  } else {
    selectedFilesSummary.textContent = 'Nenhum arquivo selecionado';
    btnSendFiles.disabled = true;
  }
});

btnSendFiles.addEventListener('click', async () => {
  if (!selectedPeerKey) {
    alert('Por favor, selecione um dispositivo de destino.');
    return;
  }
  if (pendingFiles.length === 0) return;

  btnSendFiles.disabled = true;
  btnSendFiles.textContent = 'Transferindo...';
  // Nota: no Tauri desktop nativo com input de arquivos podemos coletar os paths
  // Se forem File web, enviamos os paths via API
  const paths = pendingFiles.map(f => f.path || f.name);
  try {
    await invoke('send_files', {
      payload: { peer_key: selectedPeerKey, paths }
    });
    addActivity(`Envio de ${paths.length} arquivo(s) concluído com sucesso!`);
    pendingFiles = [];
    selectedFilesSummary.textContent = 'Nenhum arquivo selecionado';
  } catch (e) {
    alert(`Erro no envio: ${e}`);
  } finally {
    btnSendFiles.disabled = true;
    btnSendFiles.textContent = 'Enviar Arquivos';
  }
});

// Activity List Helper
function addActivity(text) {
  const empty = activityList.querySelector('.activity-empty');
  if (empty) empty.remove();

  const item = document.createElement('div');
  item.className = 'activity-item';
  item.innerHTML = `<span>${text}</span> <small style="color: var(--text-dim);">${new Date().toLocaleTimeString()}</small>`;
  activityList.prepend(item);
}

// Modal de Aceite / Recebimento
function showIncomingModal(session_id, sender, files) {
  activeSessionId = session_id;
  incomingSender.textContent = `De: ${sender.alias} (${sender.deviceModel || 'Dispositivo'})`;
  incomingFilesList.innerHTML = '';
  files.forEach(f => {
    const div = document.createElement('div');
    div.textContent = `• ${f.fileName} (${(f.size / 1024 / 1024).toFixed(2)} MB)`;
    incomingFilesList.appendChild(div);
  });
  incomingProgressBox.style.display = 'none';
  modalActionButtons.style.display = 'flex';
  incomingModal.style.display = 'grid';
}

btnAcceptTransfer.addEventListener('click', async () => {
  if (!activeSessionId) return;
  modalActionButtons.style.display = 'none';
  incomingProgressBox.style.display = 'block';
  incomingProgressText.textContent = 'Aceito! Recebendo arquivos...';
  await invoke('respond_request', { sessionId: activeSessionId, accept: true });
});

btnRejectTransfer.addEventListener('click', async () => {
  if (!activeSessionId) return;
  await invoke('respond_request', { sessionId: activeSessionId, accept: false });
  incomingModal.style.display = 'none';
  activeSessionId = null;
});

// Eventos vindos do Core Rust via Tauri
if (hasTauri) {
  window.__TAURI__.event.listen('ducker://event', (e) => {
    const ev = e.payload;
    console.log('[Ducker Core Event]', ev);
    if (ev.type === 'peerDiscovered') {
      refreshPeers();
    } else if (ev.type === 'incomingRequest') {
      if (!ev.autoAccepted) {
        showIncomingModal(ev.sessionId, ev.sender, ev.files);
      } else {
        addActivity(`Recebendo automaticamente ${ev.files.length} arquivo(s) de ${ev.sender.alias}`);
      }
    } else if (ev.type === 'textReceived') {
      addActivity(`✉️ De ${ev.sender.alias}: "${ev.text}"`);
      alert(`Mensagem recebida de ${ev.sender.alias}:\n\n"${ev.text}"`);
    } else if (ev.type === 'receiveProgress') {
      const pct = Math.min(100, Math.round((ev.received / ev.total) * 100));
      incomingProgressBar.style.width = `${pct}%`;
      incomingProgressText.textContent = `Recebendo: ${pct}%`;
    } else if (ev.type === 'fileReceived') {
      addActivity(`✓ Arquivo salvo: ${ev.fileName}`);
    } else if (ev.type === 'sessionFinished') {
      incomingModal.style.display = 'none';
      activeSessionId = null;
      addActivity(`🎉 Transferência finalizada com sucesso!`);
    } else if (ev.type === 'sessionCancelled') {
      incomingModal.style.display = 'none';
      activeSessionId = null;
      addActivity(`Transferência cancelada`);
    }
  });

  window.__TAURI__.event.listen('ducker://send-progress', (e) => {
    const p = e.payload;
    const pct = Math.min(100, Math.round((p.totalSent / p.total) * 100));
    btnSendFiles.textContent = `Enviando: ${pct}%`;
  });
}

// Inicialização
loadIdentity();
refreshPeers();
setInterval(refreshPeers, 10000);
