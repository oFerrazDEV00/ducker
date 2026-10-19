// Ducker — LocalSend v2 Protocol Integration & Interactive UI
const hasTauri = typeof window !== 'undefined' && window.__TAURI__ !== undefined;

// Mock Duckive state for standalone browser preview
let mockDuckiveView = {
  name: "Duckive",
  level: {
    level: 1,
    title: "Pato Recruta",
    currentFiles: 0,
    currentLevelMinFiles: 0,
    nextLevelFiles: 10,
    progressPct: 0.0,
    nextRewardItem: {
      id: "cap_ducker",
      name: "Boné Ducker",
      description: "Boné esportivo com aba curva para manter o foco na transferência.",
      category: "head",
      rarity: "common",
      requirement: { type: "filesSent", value: 10 },
      iconEmoji: "🧢",
      assetId: "cap_ducker"
    }
  },
  filesSentCount: 0,
  filesReceivedCount: 0,
  totalBytesSent: 0,
  totalBytesReceived: 0,
  transfersCount: 0,
  equipped: {
    head: null,
    outfit: null,
    accessory: null,
    effect: null
  },
  items: [
    {
      item: { id: "default_duck", name: "Duckive Clássico", description: "O pato original do Ducker em sua forma pura e amarela.", category: "effect", rarity: "common", requirement: { type: "defaultUnlocked" }, iconEmoji: "🦆", assetId: "default_duck" },
      unlocked: true,
      equipped: false,
      progressCurrent: 1,
      progressRequired: 1,
      progressPct: 100
    },
    {
      item: { id: "cap_ducker", name: "Boné Ducker", description: "Boné esportivo com aba curva para manter o foco na transferência.", category: "head", rarity: "common", requirement: { type: "filesSent", value: 10 }, iconEmoji: "🧢", assetId: "cap_ducker" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 10,
      progressPct: 0
    },
    {
      item: { id: "sunglasses_cool", name: "Óculos Escuros Maneiros", description: "Estilo puro e proteção contra o brilho dos pacotes de rede.", category: "accessory", rarity: "rare", requirement: { type: "filesSent", value: 25 }, iconEmoji: "🕶️", assetId: "sunglasses_cool" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 25,
      progressPct: 0
    },
    {
      item: { id: "hoodie_ducker", name: "Moletom Amarelo Ducker", description: "Conforto premium para transferências em noites frias de código.", category: "outfit", rarity: "rare", requirement: { type: "filesSent", value: 50 }, iconEmoji: "🧥", assetId: "hoodie_ducker" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 50,
      progressPct: 0
    },
    {
      item: { id: "business_suit", name: "Terno Executivo", description: "Para quando suas transferências P2P exigem seriedade corporativa.", category: "outfit", rarity: "epic", requirement: { type: "filesSent", value: 100 }, iconEmoji: "👔", assetId: "business_suit" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 100,
      progressPct: 0
    },
    {
      item: { id: "dev_coder", name: "Roupa de Programador", description: "Camiseta preta estampada com 'while(alive) { quac(); }' e fones de ouvido.", category: "outfit", rarity: "epic", requirement: { type: "filesSent", value: 250 }, iconEmoji: "💻", assetId: "dev_coder" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 250,
      progressPct: 0
    },
    {
      item: { id: "astronaut_suit", name: "Traje Espacial Ducker", description: "Blindagem pressurizada para enviar pacotes até a órbita lunar.", category: "outfit", rarity: "epic", requirement: { type: "filesSent", value: 500 }, iconEmoji: "🚀", assetId: "astronaut_suit" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 500,
      progressPct: 0
    },
    {
      item: { id: "astronaut_helmet", name: "Capacete de Astronauta", description: "Visor espelhado dourado resistente ao vácuo intergaláctico.", category: "head", rarity: "epic", requirement: { type: "filesSent", value: 500 }, iconEmoji: "👨‍🚀", assetId: "astronaut_helmet" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 500,
      progressPct: 0
    },
    {
      item: { id: "legendary_crown", name: "Coroa Real Dourada", description: "Feita em ouro maciço Quac para o soberano absoluto da rede local.", category: "head", rarity: "legendary", requirement: { type: "filesSent", value: 1000 }, iconEmoji: "👑", assetId: "legendary_crown" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 1000,
      progressPct: 0
    },
    {
      item: { id: "legendary_cape", name: "Manto Lendário Real", description: "Capa aveludada vermelha e dourada digna de lendas da computação.", category: "outfit", rarity: "legendary", requirement: { type: "filesSent", value: 1000 }, iconEmoji: "✨", assetId: "legendary_cape" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 1000,
      progressPct: 0
    },
    {
      item: { id: "cosmic_duck", name: "Aura Cósmica Suprema", description: "Emanação de energia astral reservada aos maiores mestres do Ducker.", category: "effect", rarity: "legendary", requirement: { type: "filesSent", value: 5000 }, iconEmoji: "🌌", assetId: "cosmic_duck" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 5000,
      progressPct: 0
    },
    {
      item: { id: "whatsapp_green_skin", name: "Pato Verde do WhatsApp", description: "O lendário Pato Verde do WhatsApp. Concedido exclusivamente aos mestres que alcançaram a marca histórica de 50.000 arquivos enviados pelo Ducker.", category: "effect", rarity: "legendary", requirement: { type: "filesSent", value: 50000 }, iconEmoji: "🟢", assetId: "whatsapp_green_skin" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 50000,
      progressPct: 0
    },
    {
      item: { id: "messenger_badge", name: "Pin Mensageiro Veloz", description: "Insígnia concedida pela primeira mensagem de texto enviada pelo Ducker.", category: "accessory", rarity: "rare", requirement: { type: "achievement", value: "whatsapp_green" }, iconEmoji: "💬", assetId: "messenger_badge" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 1,
      progressPct: 0
    },
    {
      item: { id: "turbo_goggles", name: "Óculos Turbo Aerodinâmicos", description: "Desenhados para reduzir o atrito em transferências acima de 5 MB/s.", category: "accessory", rarity: "rare", requirement: { type: "achievement", value: "turbo_duck" }, iconEmoji: "⚡", assetId: "turbo_goggles" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 1,
      progressPct: 0
    },
    {
      item: { id: "globe_trotter_badge", name: "Pin Pato Multiplataforma", description: "Insígnia concedida por cruzar as fronteiras entre diferentes SOs.", category: "accessory", rarity: "rare", requirement: { type: "achievement", value: "cross_platform" }, iconEmoji: "🌐", assetId: "globe_trotter_badge" },
      unlocked: false,
      equipped: false,
      progressCurrent: 0,
      progressRequired: 1,
      progressPct: 0
    }
  ],
  achievements: [
    { id: "first_flight", title: "Primeiro Voo", description: "Completou a primeira transferência de arquivos com sucesso.", iconEmoji: "🦆", unlocked: false },
    { id: "cross_platform", title: "Pato Multiplataforma", description: "Transferiu arquivos entre sistemas operacionais diferentes.", iconEmoji: "🌐", unlocked: false },
    { id: "turbo_duck", title: "Pato Turbo", description: "Atingiu velocidade superior a 5 MB/s em uma transferência.", iconEmoji: "⚡", unlocked: false },
    { id: "whatsapp_green", title: "Pato Verde do WhatsApp", description: "Enviou uma mensagem de texto rápida pelo Ducker.", iconEmoji: "💬", unlocked: false },
    { id: "centurion", title: "Centurião dos Arquivos", description: "Alcançou a marca histórica de 100 arquivos enviados.", iconEmoji: "💯", unlocked: false },
    { id: "stylist", title: "Pato Estiloso", description: "Personalizou e equipou seu primeiro cosmético no Duckive.", iconEmoji: "🎩", unlocked: false }
  ]
};

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
  if (cmd === 'get_duckive_view') {
    return mockDuckiveView;
  }
  if (cmd === 'equip_duckive_item') {
    const item = mockDuckiveView.items.find(i => i.item.id === args.itemId);
    if (item && item.unlocked) {
      const cat = item.item.category;
      if (mockDuckiveView.equipped[cat] === item.item.id) {
        mockDuckiveView.equipped[cat] = null;
      } else {
        mockDuckiveView.equipped[cat] = item.item.id;
      }
      mockDuckiveView.items.forEach(it => {
        it.equipped = mockDuckiveView.equipped[it.item.category] === it.item.id;
      });
    }
    return mockDuckiveView;
  }
  if (cmd === 'unequip_duckive_slot') {
    mockDuckiveView.equipped[args.category] = null;
    mockDuckiveView.items.forEach(it => {
      it.equipped = mockDuckiveView.equipped[it.item.category] === it.item.id;
    });
    return mockDuckiveView;
  }
  if (cmd === 'set_duckive_name') {
    mockDuckiveView.name = args.name;
    return mockDuckiveView;
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
  if (cmd === 'get_admin_session') {
    return mockAdminSession;
  }
  if (cmd === 'auth_admin') {
    if (args.code && args.code.trim() === '83578989b') {
      mockAdminSession = {
        isAdmin: true,
        adminName: 'Faelindo',
        role: 'Administrador / Faelindo',
        isDevMode: true,
        permissions: {
          canViewStats: true,
          canViewDevices: true,
          canManageCosmetics: true,
          canTestUnlocks: true,
          canViewLogs: true,
          canRunDiagnostics: true,
          canDevConfig: true
        }
      };
      return mockAdminSession;
    }
    throw new Error('Código de acesso de desenvolvedor inválido');
  }
  if (cmd === 'logout_admin') {
    mockAdminSession = null;
    return null;
  }
  if (cmd === 'admin_test_unlock') {
    const item = mockDuckiveView.items.find(i => i.item.id === args.itemId);
    if (item) {
      item.unlocked = true;
      item.progressCurrent = item.progressRequired;
      item.progressPct = 100;
    }
    return mockDuckiveView;
  }
  if (cmd === 'admin_simulate_transfer') {
    mockDuckiveView.filesSentCount += args.filesCount;
    mockDuckiveView.totalBytesSent += args.bytes;
    mockDuckiveView.transfersCount += 1;
    const newly = [];
    if (mockDuckiveView.filesSentCount >= 50000) {
      const wa = mockDuckiveView.items.find(i => i.item.id === 'whatsapp_green_skin');
      if (wa && !wa.unlocked) {
        wa.unlocked = true;
        wa.progressCurrent = 50000;
        wa.progressPct = 100;
        newly.push(wa.item);
      }
    }
    return {
      newlyUnlockedItems: newly,
      newlyUnlockedAchievements: [],
      totalFilesSent: mockDuckiveView.filesSentCount,
      levelUp: true,
      currentLevel: mockDuckiveView.filesSentCount >= 50000 ? 10 : 8
    };
  }
  if (cmd === 'admin_get_stats') {
    return {
      is_debug: true,
      peer_count: currentPeers.length || 3,
      port: 53317,
      save_dir: 'C:/Users/Rafael/Downloads',
      total_files_sent: mockDuckiveView.filesSentCount,
      total_bytes_sent: mockDuckiveView.totalBytesSent,
      total_files_received: mockDuckiveView.filesReceivedCount,
      total_bytes_received: mockDuckiveView.totalBytesReceived,
      transfers_count: mockDuckiveView.transfersCount,
      duckive_level: mockDuckiveView.level.level,
      duckive_level_title: mockDuckiveView.level.title,
      items_unlocked_count: mockDuckiveView.items.filter(i => i.unlocked).length,
      total_items_count: mockDuckiveView.items.length
    };
  }
  if (cmd === 'check_for_updates') {
    return {
      available: false,
      currentVersion: '0.2.0',
      latestVersion: '0.2.0',
      releaseName: 'v0.2.0',
      releaseNotes: 'Versão de desenvolvimento local.',
      publishedAt: '2026-10-19',
      assetName: null,
      assetSize: null,
      assetUrl: null,
      statusMessage: 'O Ducker já está na versão mais recente (v0.2.0). 🦆'
    };
  }
  if (cmd === 'download_update') {
    return 'C:/fake/path/ducker-setup.exe';
  }
  if (cmd === 'cancel_update_download' || cmd === 'install_update') {
    return null;
  }
  return null;
};

let mockAdminSession = null;

// State
let currentPeers = [];
let selectedPeerKey = null;
let activeSessionId = null;
let pendingFiles = [];
let myIdentity = null;

// Chat State & Storage
const CHAT_STORAGE_KEY = 'ducker_chat_history_v1';
let activeChatPeerKey = null;
let chatConversations = loadChatHistory();
let audioCtx = null;

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

// DOM Elements - Chat
const chatSearchInput = document.getElementById('chatSearchInput');
const chatContactsList = document.getElementById('chatContactsList');
const chatEmptyState = document.getElementById('chatEmptyState');
const chatActiveView = document.getElementById('chatActiveView');
const chatActiveAvatar = document.getElementById('chatActiveAvatar');
const chatActiveName = document.getElementById('chatActiveName');
const chatActiveSubtitle = document.getElementById('chatActiveSubtitle');
const chatMessagesStream = document.getElementById('chatMessagesStream');
const chatInputText = document.getElementById('chatInputText');
const btnChatSend = document.getElementById('btnChatSend');
const btnChatBack = document.getElementById('btnChatBack');
const btnChatAttachFile = document.getElementById('btnChatAttachFile');
const btnChatSendFileDirect = document.getElementById('btnChatSendFileDirect');
const btnChatQuickDuck = document.getElementById('btnChatQuickDuck');
const btnChatClearHistory = document.getElementById('btnChatClearHistory');

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

  if (pageName === 'chat') {
    renderChatContacts(chatSearchInput ? chatSearchInput.value : '');
    if (activeChatPeerKey) {
      renderChatMessages();
    } else {
      const layoutBox = document.querySelector('.chat-layout-box');
      if (layoutBox) layoutBox.classList.remove('mobile-in-chat');
    }
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
document.getElementById('btnHeaderDuckive')?.addEventListener('click', () => navigateToPage('duckive'));
document.getElementById('btnMobileDuckive')?.addEventListener('click', () => navigateToPage('duckive'));
document.getElementById('btnOpenDuckiveFromHome')?.addEventListener('click', () => navigateToPage('duckive'));

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
          <h5>${escapeHtml(peer.info.alias || 'Dispositivo')}</h5>
          <p>${peer.ip}</p>
        </div>
      </div>
      <div style="display:flex; align-items:center; gap:8px;">
        <button class="btn-quick-chat" title="Abrir conversa no Chat">💬 Chat</button>
        <div class="badge-online">
          <span class="dot"></span>
          <span>Online</span>
        </div>
      </div>
    `;
    card.addEventListener('click', () => {
      selectedPeerKey = peerKey;
      renderDevices(currentPeers);
    });
    card.querySelector('.btn-quick-chat')?.addEventListener('click', (e) => {
      e.stopPropagation();
      openChatWithPeer(peerKey);
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
            <h4 style="font-weight:700;">${escapeHtml(peer.info.alias)}</h4>
            <span style="font-size:0.8rem; color:var(--text-dim);">${escapeHtml(peer.info.deviceModel || 'Modelo padrão')}</span>
          </div>
        </div>
        <div class="badge-online"><span class="dot"></span> Online</div>
      </div>
      <div style="font-size:0.82rem; color:var(--text-muted); display:flex; flex-direction:column; gap:4px; font-family:var(--font-mono); margin-top:8px;">
        <div>IP: ${peer.ip}:${peer.port} (${peer.protocol.toUpperCase()})</div>
        <div>FP: ${peer.info.fingerprint.slice(0, 16)}...</div>
      </div>
      <div style="display:flex; gap:8px; margin-top:8px;">
        <button class="btn btn-secondary btn-sm btn-action-select" style="flex:1;">Selecionar para Envio</button>
        <button class="btn btn-accent-yellow btn-sm btn-action-chat" style="flex:1;">💬 Conversar</button>
      </div>
    `;
    detCard.querySelector('.btn-action-select').addEventListener('click', () => {
      selectedPeerKey = peerKey;
      navigateToPage('home');
      renderDevices(currentPeers);
    });
    detCard.querySelector('.btn-action-chat').addEventListener('click', () => {
      openChatWithPeer(peerKey);
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
  renderChatContacts(chatSearchInput ? chatSearchInput.value : '');
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
    
    // Sincroniza envio com o histórico de chat
    const lowerKey = selectedPeerKey.toLowerCase();
    if (!chatConversations[lowerKey]) {
      const peer = currentPeers.find(p => (p.key || p.info.fingerprint || `${p.ip}:${p.port}`).toLowerCase() === lowerKey);
      chatConversations[lowerKey] = {
        peerInfo: {
          alias: peer ? peer.info.alias : 'Dispositivo',
          fingerprint: peer ? peer.info.fingerprint : lowerKey,
          deviceModel: peer ? peer.info.deviceModel : '',
          deviceType: peer ? peer.info.deviceType : 'desktop',
          ip: peer ? peer.ip : ''
        },
        unread: 0,
        lastTimestamp: Date.now(),
        messages: []
      };
    }
    chatConversations[lowerKey].messages.push({
      id: 'msg_' + Date.now(),
      direction: 'sent',
      text,
      timestamp: Date.now(),
      status: 'sent'
    });
    chatConversations[lowerKey].lastTimestamp = Date.now();
    saveChatHistory();
    playMessageChime(true);
    renderChatContacts(chatSearchInput ? chatSearchInput.value : '');

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

// ==================== DUCKIVE GAMIFICATION SYSTEM ====================
let currentDuckiveView = null;
let currentActiveTab = 'all';
let pendingCelebrationItem = null;

function renderDuckiveSvg(equipped = {}, size = 120) {
  const isGreen = equipped.effect === 'whatsapp_green_skin';
  const bodyColor = isGreen ? '#25D366' : '#F7C844';
  const bodyShade = isGreen ? '#1EBE5D' : '#E5B630';
  const beakColor = '#FF8C00';
  const eyeColor = '#111827';

  let svg = `<svg viewBox="0 0 100 100" width="${size}" height="${size}" xmlns="http://www.w3.org/2000/svg">`;

  // 1. EFEITO DE FUNDO: Aura Cósmica ou Pato Verde do WhatsApp
  if (isGreen) {
    svg += `
      <defs>
        <radialGradient id="waGreenGlow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stop-color="#25D366" stop-opacity="0.8"/>
          <stop offset="65%" stop-color="#128C7E" stop-opacity="0.35"/>
          <stop offset="100%" stop-color="#25D366" stop-opacity="0"/>
        </radialGradient>
      </defs>
      <circle cx="50" cy="52" r="44" fill="url(#waGreenGlow)" />
      <circle cx="18" cy="24" r="2.2" fill="#25D366" opacity="0.9"/>
      <circle cx="82" cy="28" r="1.8" fill="#34D399" opacity="0.9"/>
      <circle cx="86" cy="68" r="2" fill="#25D366" opacity="0.8"/>
      <circle cx="14" cy="62" r="1.6" fill="#A7F3D0" opacity="0.8"/>
    `;
  } else if (equipped.effect === 'cosmic_duck') {
    svg += `
      <defs>
        <radialGradient id="cosmicGlow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stop-color="#a855f7" stop-opacity="0.85"/>
          <stop offset="60%" stop-color="#ec4899" stop-opacity="0.4"/>
          <stop offset="100%" stop-color="#3b82f6" stop-opacity="0"/>
        </radialGradient>
      </defs>
      <circle cx="50" cy="52" r="44" fill="url(#cosmicGlow)" />
      <circle cx="18" cy="24" r="2.2" fill="#ffffff" opacity="0.9"/>
      <circle cx="82" cy="28" r="1.8" fill="#fef08a" opacity="0.9"/>
      <circle cx="86" cy="68" r="2" fill="#67e8f9" opacity="0.8"/>
      <circle cx="14" cy="62" r="1.6" fill="#f472b6" opacity="0.8"/>
    `;
  }

  // 1.5 ROUPA DE FUNDO: Capa Real Lendária
  if (equipped.outfit === 'legendary_cape') {
    svg += `
      <path d="M 38 48 C 24 50 16 68 18 86 C 26 88 38 84 43 78 Z" fill="#DC2626" />
      <path d="M 38 48 C 28 52 22 68 23 84" stroke="#FBBF24" stroke-width="2" fill="none"/>
    `;
  }

  // 2. CORPO BASE DO PATO DUCKER
  svg += `
    <!-- Corpo -->
    <path d="M 35 48 C 35 32 46 20 62 20 C 76 20 86 30 86 44 C 86 52 82 58 76 62 C 78 68 83 72 88 74 C 86 78 78 80 70 78 C 55 82 35 76 35 48 Z" fill="${bodyColor}" />
    <!-- Asa -->
    <path d="M 40 52 C 40 44 48 40 58 42 C 64 43 68 48 66 58 C 64 68 48 70 42 66 C 39 63 40 56 40 52 Z" fill="${bodyShade}" />
    <!-- Penas da cauda -->
    <path d="M 86 72 C 92 68 95 62 96 58 C 93 64 88 68 84 70 Z" fill="${bodyShade}" />
    <!-- Bochecha corada -->
    <ellipse cx="62" cy="38" rx="4" ry="2.5" fill="#FF7043" opacity="0.35" />
    <!-- Bico -->
    <path d="M 75 32 C 84 32 93 35 95 38 C 93 42 82 44 75 42 Z" fill="${beakColor}" />
    <circle cx="82" cy="35" r="0.8" fill="#B45309" />
    <!-- Olho carismático -->
    <ellipse cx="68" cy="28" rx="3.5" ry="4" fill="${eyeColor}" />
    <circle cx="69.2" cy="26.8" r="1.3" fill="#FFFFFF" />
  `;

  // 3. CAMADA DE ROUPA (Frente)
  if (equipped.outfit === 'hoodie_ducker') {
    svg += `
      <!-- Moletom Amarelo Ducker -->
      <path d="M 38 48 C 42 38 52 38 58 42 C 65 44 68 50 67 60 C 65 72 45 74 38 64 C 36 60 37 52 38 48 Z" fill="#EAB308" stroke="#CA8A04" stroke-width="1.5"/>
      <path d="M 46 48 L 46 58" stroke="#FFFFFF" stroke-width="1.2" stroke-linecap="round"/>
      <path d="M 52 49 L 52 57" stroke="#FFFFFF" stroke-width="1.2" stroke-linecap="round"/>
      <ellipse cx="50" cy="62" rx="6" ry="3.5" fill="#CA8A04" opacity="0.5"/>
    `;
  } else if (equipped.outfit === 'business_suit') {
    svg += `
      <!-- Terno Executivo -->
      <path d="M 39 50 C 44 42 54 42 60 45 C 66 48 68 54 66 62 C 63 72 46 72 40 65 Z" fill="#1E293B" />
      <polygon points="50,45 54,45 52,56" fill="#FFFFFF" />
      <polygon points="51,46 53,46 53.5,58 52,60 50.5,58" fill="#F59E0B" />
    `;
  } else if (equipped.outfit === 'dev_coder') {
    svg += `
      <!-- Camiseta Dev Coder -->
      <path d="M 39 49 C 44 42 54 42 60 45 C 66 48 68 54 66 62 C 63 72 46 72 40 65 Z" fill="#0F172A" />
      <text x="44" y="58" font-family="monospace" font-size="4.2" font-weight="bold" fill="#22C55E">&lt;quac/&gt;</text>
      <!-- Fones de Ouvido no pescoço -->
      <path d="M 46 42 C 40 40 38 34 38 30" stroke="#38BDF8" stroke-width="2.5" fill="none" stroke-linecap="round"/>
      <circle cx="38" cy="30" r="3.5" fill="#0284C7"/>
    `;
  } else if (equipped.outfit === 'astronaut_suit') {
    svg += `
      <!-- Traje Espacial Astronauta -->
      <path d="M 38 48 C 42 38 52 38 60 42 C 67 44 70 52 68 64 C 65 74 44 75 38 64 Z" fill="#F1F5F9" stroke="#94A3B8" stroke-width="1.5"/>
      <rect x="47" y="52" width="8" height="6" rx="1.5" fill="#0284C7" />
      <circle cx="49" cy="55" r="1" fill="#22C55E" />
      <circle cx="53" cy="55" r="1" fill="#EF4444" />
    `;
  }

  // 4. CAMADA DE ACESSÓRIOS
  if (equipped.accessory === 'sunglasses_cool') {
    svg += `
      <!-- Óculos Escuros Maneiros -->
      <path d="M 62 26 L 76 27 L 74 34 L 62 33 Z" fill="#09090B" />
      <path d="M 76 27 L 84 29 L 81 35 L 74 34 Z" fill="#09090B" />
      <line x1="57" y1="28" x2="62" y2="28" stroke="#09090B" stroke-width="2" />
      <line x1="64" y1="28" x2="72" y2="33" stroke="#FFFFFF" stroke-width="1" opacity="0.6"/>
      <line x1="77" y1="29" x2="81" y2="33" stroke="#FFFFFF" stroke-width="1" opacity="0.6"/>
    `;
  } else if (equipped.accessory === 'turbo_goggles') {
    svg += `
      <!-- Óculos Turbo -->
      <ellipse cx="68" cy="29" rx="6" ry="5.5" fill="#FDE047" stroke="#EA580C" stroke-width="2" />
      <ellipse cx="80" cy="32" rx="5" ry="5" fill="#FDE047" stroke="#EA580C" stroke-width="2" />
      <line x1="56" y1="27" x2="63" y2="28" stroke="#EA580C" stroke-width="2.5" />
      <line x1="74" y1="30" x2="76" y2="31" stroke="#EA580C" stroke-width="2" />
    `;
  } else if (equipped.accessory === 'globe_trotter_badge') {
    svg += `
      <!-- Pin Multiplataforma -->
      <circle cx="46" cy="58" r="4.5" fill="#0284C7" stroke="#F59E0B" stroke-width="1.2" />
      <path d="M 43 58 C 45 56 47 56 49 58 C 47 60 45 60 43 58" fill="#22C55E" />
    `;
  }

  // 5. CAMADA DE CABEÇA
  if (equipped.head === 'cap_ducker') {
    svg += `
      <!-- Boné Ducker -->
      <path d="M 52 20 C 53 11 72 10 75 19 Z" fill="#1E293B" />
      <path d="M 68 18 C 76 18 86 16 92 19 C 88 23 78 22 72 20 Z" fill="#F59E0B" />
      <circle cx="63" cy="13" r="1.5" fill="#F59E0B" />
    `;
  } else if (equipped.head === 'astronaut_helmet') {
    svg += `
      <!-- Capacete Astronauta -->
      <circle cx="68" cy="30" r="18" fill="rgba(186, 230, 253, 0.35)" stroke="#CBD5E1" stroke-width="2" />
      <path d="M 62 20 C 74 18 82 25 82 35" stroke="#F59E0B" stroke-width="3" stroke-linecap="round" fill="none" opacity="0.85" />
      <path d="M 56 22 C 58 17 64 15 69 15" stroke="#FFFFFF" stroke-width="2" stroke-linecap="round" fill="none" opacity="0.9" />
    `;
  } else if (equipped.head === 'legendary_crown') {
    svg += `
      <!-- Coroa Real Dourada -->
      <path d="M 56 18 L 54 9 L 63 13 L 70 6 L 77 13 L 84 9 L 82 18 Z" fill="#F59E0B" stroke="#B45309" stroke-width="1.2"/>
      <circle cx="70" cy="9" r="1.8" fill="#EF4444" />
      <circle cx="63" cy="15" r="1.2" fill="#3B82F6" />
      <circle cx="77" cy="15" r="1.2" fill="#10B981" />
      <line x1="56" y1="17" x2="82" y2="17" stroke="#FDE047" stroke-width="1.5" />
    `;
  }

  svg += `</svg>`;
  return svg;
}

// Carrega dados completos do Duckive
async function loadDuckive() {
  try {
    const view = await invoke('get_duckive_view');
    if (view) {
      updateDuckiveUI(view);
    }
  } catch (err) {
    console.error('Falha ao carregar Duckive:', err);
  }
}

// Atualiza toda a interface visual onde o Duckive aparece
function updateDuckiveUI(view) {
  if (!view) return;
  currentDuckiveView = view;

  // 1. Sidebar desktop
  const sidebarBadge = document.getElementById('sidebarDuckiveBadge');
  if (sidebarBadge) {
    sidebarBadge.textContent = `Nv. ${view.level.level}`;
  }

  // 2. Header desktop
  const headerAvatar = document.getElementById('headerDuckiveAvatar');
  if (headerAvatar) {
    headerAvatar.innerHTML = renderDuckiveSvg(view.equipped, 30);
  }
  const headerName = document.getElementById('headerDuckiveName');
  if (headerName) {
    headerName.textContent = view.name || 'Duckive';
  }
  const headerLvl = document.getElementById('headerDuckiveLvl');
  if (headerLvl) {
    headerLvl.textContent = `Nível ${view.level.level}`;
  }

  // 3. Header mobile
  const mobileWrap = document.getElementById('mobileDuckiveAvatarWrap');
  if (mobileWrap) {
    mobileWrap.innerHTML = renderDuckiveSvg(view.equipped, 24);
  }
  const mobileLvl = document.getElementById('mobileDuckiveLvlBadge');
  if (mobileLvl) {
    mobileLvl.textContent = `Nv. ${view.level.level}`;
  }

  // 4. Card companheiro na Home
  const homeAvatar = document.getElementById('duckiveHomeAvatar');
  if (homeAvatar) {
    homeAvatar.innerHTML = renderDuckiveSvg(view.equipped, 52);
  }
  const homeName = document.getElementById('duckiveHomeName');
  if (homeName) {
    homeName.textContent = view.name || 'Meu Duckive';
  }
  const homeBadge = document.getElementById('duckiveHomeBadge');
  if (homeBadge) {
    homeBadge.textContent = `Nível ${view.level.level} • ${view.level.title}`;
  }
  const homeNextUnlock = document.getElementById('duckiveHomeNextUnlock');
  if (homeNextUnlock) {
    if (view.level.nextRewardItem) {
      homeNextUnlock.textContent = `Próximo: ${view.level.nextRewardItem.name} (${view.filesSentCount}/${view.level.nextLevelFiles} arquivos)`;
    } else {
      homeNextUnlock.textContent = `Nível Máximo Alcançado! 🌌`;
    }
  }
  const homeFill = document.getElementById('duckiveHomeProgressFill');
  if (homeFill) {
    homeFill.style.width = `${view.level.progressPct}%`;
  }

  // 5. Página dedicada do Duckive
  const stageAvatar = document.getElementById('duckiveStageAvatarWrapper');
  if (stageAvatar) {
    stageAvatar.innerHTML = renderDuckiveSvg(view.equipped, 160);
  }
  const stageName = document.getElementById('duckiveStageName');
  if (stageName) {
    stageName.textContent = view.name || 'Duckive';
  }
  const stageLevelTitle = document.getElementById('duckiveStageLevelTitle');
  if (stageLevelTitle) {
    stageLevelTitle.textContent = `Nível ${view.level.level} • ${view.level.title}`;
  }
  const stagePct = document.getElementById('duckiveProgPct');
  if (stagePct) {
    stagePct.textContent = `${Math.round(view.level.progressPct)}%`;
  }
  const stageFill = document.getElementById('duckiveStageProgressFill');
  if (stageFill) {
    stageFill.style.width = `${view.level.progressPct}%`;
  }
  const stageFilesCounter = document.getElementById('duckiveStageFilesCounter');
  if (stageFilesCounter) {
    stageFilesCounter.textContent = `${view.filesSentCount} / ${view.level.nextLevelFiles || view.filesSentCount} arquivos`;
  }
  const stageReward = document.getElementById('duckiveStageNextReward');
  if (stageReward) {
    stageReward.textContent = view.level.nextRewardItem
      ? `Próximo: ${view.level.nextRewardItem.name}`
      : `Mestre Supremo dos Quacs!`;
  }

  // Stats
  const statFiles = document.getElementById('duckiveStatFilesSent');
  if (statFiles) statFiles.textContent = view.filesSentCount;
  const statTransfers = document.getElementById('duckiveStatTransfers');
  if (statTransfers) statTransfers.textContent = view.transfersCount;
  const statAch = document.getElementById('duckiveStatAchievements');
  if (statAch) {
    const unl = view.achievements.filter(a => a.unlocked).length;
    statAch.textContent = `${unl}/${view.achievements.length}`;
  }

  // Chips dos itens equipados
  renderEquippedChips(view);

  // Inventário e Conquistas
  renderCosmeticsGrid(view.items, currentActiveTab);
  renderAchievementsGrid(view.achievements);
}

// Renderiza chips com os itens equipados no card da esquerda
function renderEquippedChips(view) {
  const container = document.getElementById('equippedChipsList');
  if (!container) return;
  container.innerHTML = '';

  const slots = [
    { cat: 'head', label: '👑 Cabeça', id: view.equipped.head },
    { cat: 'outfit', label: '🧥 Roupa', id: view.equipped.outfit },
    { cat: 'accessory', label: '👓 Acessório', id: view.equipped.accessory },
    { cat: 'effect', label: '✨ Efeito', id: view.equipped.effect }
  ];

  let hasAny = false;
  slots.forEach(slot => {
    if (slot.id) {
      hasAny = true;
      const it = view.items.find(i => i.item.id === slot.id);
      const name = it ? it.item.name : slot.id;
      const chip = document.createElement('div');
      chip.className = 'equipped-chip';
      chip.innerHTML = `
        <span>${slot.label}: <strong>${name}</strong></span>
        <button class="equipped-chip-btn-remove" title="Desequipar" data-slot="${slot.cat}">×</button>
      `;
      chip.querySelector('.equipped-chip-btn-remove').addEventListener('click', async (e) => {
        e.stopPropagation();
        await unequipDuckiveSlot(slot.cat);
      });
      container.appendChild(chip);
    }
  });

  if (!hasAny) {
    container.innerHTML = `<span style="font-size: 0.78rem; color: var(--text-dim);">Nenhum item equipado no momento.</span>`;
  }
}

// Renderiza o grid de cosméticos filtrado
function renderCosmeticsGrid(items, filterCategory) {
  const grid = document.getElementById('cosmeticsItemsGrid');
  if (!grid) return;
  grid.innerHTML = '';

  const filtered = items.filter(it => {
    if (filterCategory === 'all') return true;
    return it.item.category.toLowerCase() === filterCategory.toLowerCase();
  });

  filtered.forEach(it => {
    const item = it.item;
    const card = document.createElement('div');
    card.className = `cosmetic-card rarity-border-${item.rarity.toLowerCase()} ${it.equipped ? 'equipped' : ''} ${!it.unlocked ? 'locked' : ''}`;

    let actionHtml = '';
    if (it.unlocked) {
      if (it.equipped) {
        actionHtml = `<button class="btn btn-secondary btn-sm btn-equip btn-toggle-equip" data-id="${item.id}">✓ Equipado (Remover)</button>`;
      } else {
        actionHtml = `<button class="btn btn-accent-yellow btn-sm btn-equip btn-toggle-equip" data-id="${item.id}">Equipar 🦆</button>`;
      }
    } else {
      let reqText = `Bloqueado`;
      if (item.requirement.type === 'filesSent') {
        reqText = `${it.progressCurrent}/${it.progressRequired} arquivos`;
      } else if (item.requirement.type === 'achievement') {
        reqText = `Requer conquista`;
      }
      actionHtml = `
        <div class="cosmetic-lock-info">
          <div class="lock-req-text">
            <span>🔒 Requisito:</span>
            <span>${reqText}</span>
          </div>
          <div class="cosmetic-mini-rail">
            <div class="cosmetic-mini-fill" style="width: ${it.progressPct}%"></div>
          </div>
        </div>
      `;
    }

    card.innerHTML = `
      <div class="cosmetic-card-top">
        <div class="cosmetic-icon-bubble">${item.iconEmoji}</div>
        <span class="cosmetic-badge-rarity rarity-${item.rarity.toLowerCase()}">${item.rarity}</span>
      </div>
      <div class="cosmetic-card-body">
        <h4>${item.name}</h4>
        <span class="cosmetic-category-tag">${item.category.toUpperCase()}</span>
        <p class="cosmetic-desc">${item.description}</p>
      </div>
      <div class="cosmetic-card-action">
        ${actionHtml}
      </div>
    `;

    card.querySelectorAll('.btn-toggle-equip').forEach(btn => {
      btn.addEventListener('click', async () => {
        await equipDuckiveItem(btn.dataset.id);
      });
    });

    grid.appendChild(card);
  });
}

// Renderiza o grid de conquistas
function renderAchievementsGrid(achievements) {
  const grid = document.getElementById('achievementsItemsGrid');
  if (!grid) return;
  grid.innerHTML = '';

  achievements.forEach(ach => {
    const card = document.createElement('div');
    card.className = `achievement-card ${ach.unlocked ? 'unlocked' : 'locked'}`;

    card.innerHTML = `
      <div class="achievement-icon-box">${ach.iconEmoji}</div>
      <div class="achievement-details">
        <h4>${ach.title}</h4>
        <p>${ach.description}</p>
        <span class="achievement-status-badge ${ach.unlocked ? 'badge-ach-unlocked' : 'badge-ach-locked'}">
          ${ach.unlocked ? '✓ Conquistado' : '🔒 Bloqueado'}
        </span>
      </div>
    `;
    grid.appendChild(card);
  });
}

// Equipa ou desequipa um item cosmético
async function equipDuckiveItem(itemId) {
  try {
    const view = await invoke('equip_duckive_item', { itemId });
    if (view) {
      updateDuckiveUI(view);
    }
  } catch (err) {
    alert(`Erro ao equipar item: ${err}`);
  }
}

// Desequipa um slot específico
async function unequipDuckiveSlot(category) {
  try {
    const view = await invoke('unequip_duckive_slot', { category });
    if (view) {
      updateDuckiveUI(view);
    }
  } catch (err) {
    alert(`Erro ao desequipar: ${err}`);
  }
}

// Abre o modal comemorativo de desbloqueio
function showCelebrationModal(summary) {
  const modal = document.getElementById('duckiveUnlockModal');
  const dialog = modal?.querySelector('.modal-dialog-celebration');
  const avatarBox = document.getElementById('celebrationAvatarPreview');
  const titleEl = document.getElementById('celebrationTitle');
  const descEl = document.getElementById('celebrationDesc');
  const btnEquip = document.getElementById('btnEquipCelebrationNow');
  const badgeTop = modal?.querySelector('.celebration-badge-top');

  if (!modal) return;
  if (dialog) dialog.classList.remove('celebration-legendary-whatsapp');
  if (badgeTop) badgeTop.textContent = '🎉 DESBLOQUEIO DUCKIVE';

  if (summary.newlyUnlockedItems && summary.newlyUnlockedItems.length > 0) {
    const item = summary.newlyUnlockedItems[0];
    pendingCelebrationItem = item.id;

    if (item.id === 'whatsapp_green_skin') {
      if (dialog) dialog.classList.add('celebration-legendary-whatsapp');
      if (badgeTop) badgeTop.textContent = '🟢 50.000 ARQUIVOS • MARCA HISTÓRICA';
      titleEl.textContent = '🦆 PATO VERDE DO WHATSAPP!';
      descEl.textContent = 'Extraordinário! Você atingiu a lendária marca de 50.000 arquivos enviados pelo Ducker! O cosmético lendário Pato Verde do WhatsApp foi desbloqueado permanentemente no seu inventário!';
      btnEquip.style.display = 'inline-block';
      btnEquip.textContent = 'Equipar Pato Verde 🟢';
    } else {
      titleEl.textContent = `Novo Item: ${item.name}!`;
      descEl.textContent = item.description;
      btnEquip.style.display = 'inline-block';
      btnEquip.textContent = `Equipar ${item.iconEmoji}`;
    }

    const tempEquipped = Object.assign({}, currentDuckiveView?.equipped || {});
    tempEquipped[item.category] = item.id;
    avatarBox.innerHTML = renderDuckiveSvg(tempEquipped, 110);
  } else if (summary.newlyUnlockedAchievements && summary.newlyUnlockedAchievements.length > 0) {
    const ach = summary.newlyUnlockedAchievements[0];
    pendingCelebrationItem = null;
    titleEl.textContent = `🏆 Conquista: ${ach.title}!`;
    descEl.textContent = ach.description;
    btnEquip.style.display = 'none';
    avatarBox.innerHTML = renderDuckiveSvg(currentDuckiveView?.equipped || {}, 110);
  } else if (summary.levelUp) {
    pendingCelebrationItem = null;
    titleEl.textContent = `⭐ Level Up: Nível ${summary.currentLevel}!`;
    descEl.textContent = `Seu Duckive continua evoluindo com as transferências locais!`;
    btnEquip.style.display = 'none';
    avatarBox.innerHTML = renderDuckiveSvg(currentDuckiveView?.equipped || {}, 110);
  } else {
    return;
  }

  modal.style.display = 'grid';
}

document.getElementById('btnDismissCelebration')?.addEventListener('click', () => {
  const modal = document.getElementById('duckiveUnlockModal');
  const dialog = modal?.querySelector('.modal-dialog-celebration');
  if (dialog) dialog.classList.remove('celebration-legendary-whatsapp');
  if (modal) modal.style.display = 'none';
  pendingCelebrationItem = null;
});

document.getElementById('btnEquipCelebrationNow')?.addEventListener('click', async () => {
  if (pendingCelebrationItem) {
    await equipDuckiveItem(pendingCelebrationItem);
  }
  const modal = document.getElementById('duckiveUnlockModal');
  const dialog = modal?.querySelector('.modal-dialog-celebration');
  if (dialog) dialog.classList.remove('celebration-legendary-whatsapp');
  if (modal) modal.style.display = 'none';
  pendingCelebrationItem = null;
});

// Abas do Duckive
document.querySelectorAll('.duckive-nav-tabs .duckive-tab-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.duckive-nav-tabs .duckive-tab-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');

    const tab = btn.dataset.tab;
    currentActiveTab = tab;

    const paneItems = document.getElementById('paneItems');
    const paneAch = document.getElementById('paneAchievements');

    if (tab === 'achievements') {
      if (paneItems) paneItems.style.display = 'none';
      if (paneAch) paneAch.style.display = 'block';
    } else {
      if (paneAch) paneAch.style.display = 'none';
      if (paneItems) paneItems.style.display = 'block';
      if (currentDuckiveView) {
        renderCosmeticsGrid(currentDuckiveView.items, tab);
      }
    }
  });
});

// Renomear Duckive
document.getElementById('btnEditDuckiveName')?.addEventListener('click', async () => {
  const current = currentDuckiveView?.name || 'Duckive';
  const newName = prompt('Digite o novo nome para o seu Duckive:', current);
  if (newName && newName.trim().length > 0) {
    try {
      const view = await invoke('set_duckive_name', { name: newName.trim() });
      if (view) {
        updateDuckiveUI(view);
      }
    } catch (e) {
      alert(`Erro ao renomear Duckive: ${e}`);
    }
  }
});

document.getElementById('btnRefreshDuckive')?.addEventListener('click', () => {
  loadDuckive();
});

// ==================== DUCKER MESSENGER (P2P CHAT) ====================

function escapeHtml(str) {
  if (!str) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function formatChatMessageText(text) {
  const escaped = escapeHtml(text);
  const urlRegex = /(https?:\/\/[^\s<]+)/g;
  return escaped.replace(urlRegex, (url) => {
    return `<a href="${url}" target="_blank" rel="noopener noreferrer">${url}</a>`;
  }).replace(/\n/g, '<br>');
}

function isToday(date) {
  const today = new Date();
  return date.getDate() === today.getDate() &&
    date.getMonth() === today.getMonth() &&
    date.getFullYear() === today.getFullYear();
}

function isYesterday(date) {
  const yesterday = new Date();
  yesterday.setDate(yesterday.getDate() - 1);
  return date.getDate() === yesterday.getDate() &&
    date.getMonth() === yesterday.getMonth() &&
    date.getFullYear() === yesterday.getFullYear();
}

function loadChatHistory() {
  try {
    const raw = localStorage.getItem(CHAT_STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch (e) {
    console.error('Falha ao carregar histórico de chat:', e);
    return {};
  }
}

function saveChatHistory() {
  try {
    localStorage.setItem(CHAT_STORAGE_KEY, JSON.stringify(chatConversations));
  } catch (e) {
    console.error('Falha ao salvar histórico de chat:', e);
  }
}

function updateChatBadges() {
  let totalUnread = 0;
  Object.values(chatConversations).forEach(c => {
    totalUnread += (c.unread || 0);
  });
  const sidebarBadge = document.getElementById('sidebarChatBadge');
  if (sidebarBadge) {
    sidebarBadge.textContent = totalUnread;
    sidebarBadge.style.display = totalUnread > 0 ? 'inline-block' : 'none';
  }
  const mobileBadge = document.getElementById('mobileChatBadge');
  if (mobileBadge) {
    mobileBadge.textContent = totalUnread;
    mobileBadge.style.display = totalUnread > 0 ? 'inline-block' : 'none';
  }
}

function playMessageChime(isSent = false) {
  try {
    if (!audioCtx) {
      audioCtx = new (window.AudioContext || window.webkitAudioContext)();
    }
    if (audioCtx.state === 'suspended') {
      audioCtx.resume();
    }
    const osc = audioCtx.createOscillator();
    const gain = audioCtx.createGain();
    osc.connect(gain);
    gain.connect(audioCtx.destination);
    const now = audioCtx.currentTime;

    if (isSent) {
      osc.frequency.setValueAtTime(587.33, now); // D5
      osc.frequency.exponentialRampToValueAtTime(880, now + 0.08); // A5
      gain.gain.setValueAtTime(0.12, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + 0.12);
      osc.start(now);
      osc.stop(now + 0.12);
    } else {
      osc.frequency.setValueAtTime(523.25, now); // C5
      osc.frequency.exponentialRampToValueAtTime(659.25, now + 0.09); // E5
      gain.gain.setValueAtTime(0.18, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + 0.18);
      osc.start(now);
      osc.stop(now + 0.18);
    }
  } catch (e) {
    // Audio Context bloqueado ou indisponível
  }
}

function showChatToast(senderAlias, text, peerKey) {
  const container = document.getElementById('toastContainer');
  if (!container) return;
  const toast = document.createElement('div');
  toast.className = 'chat-toast';
  toast.innerHTML = `
    <div class="chat-toast-avatar">🦆</div>
    <div class="chat-toast-content">
      <div class="chat-toast-title">✉️ ${escapeHtml(senderAlias || 'Dispositivo')}</div>
      <div class="chat-toast-text">${escapeHtml(text || '')}</div>
    </div>
    <button class="chat-toast-action">Ver</button>
    <button class="chat-toast-close" title="Fechar">✕</button>
  `;
  toast.querySelector('.chat-toast-action').addEventListener('click', () => {
    openChatWithPeer(peerKey);
    toast.remove();
  });
  toast.querySelector('.chat-toast-close').addEventListener('click', () => {
    toast.remove();
  });
  container.appendChild(toast);
  setTimeout(() => {
    toast.style.opacity = '0';
    toast.style.transform = 'translateY(-10px)';
    setTimeout(() => toast.remove(), 300);
  }, 6000);
}

function renderChatContacts(filterText = '') {
  if (!chatContactsList) return;

  const allKeys = new Set();
  const peerMap = {};

  currentPeers.forEach(p => {
    const key = (p.key || p.info.fingerprint || `${p.ip}:${p.port}`).toLowerCase();
    allKeys.add(key);
    peerMap[key] = {
      online: true,
      peer: p,
      alias: p.info.alias,
      model: p.info.deviceModel,
      type: p.info.deviceType,
      ip: p.ip,
      port: p.port,
      fingerprint: p.info.fingerprint,
      key: p.key || key
    };
  });

  Object.keys(chatConversations).forEach(k => {
    const lower = k.toLowerCase();
    allKeys.add(lower);
    if (!peerMap[lower]) {
      const saved = chatConversations[k].peerInfo || {};
      peerMap[lower] = {
        online: false,
        peer: null,
        alias: saved.alias || 'Dispositivo',
        model: saved.deviceModel,
        type: saved.deviceType,
        ip: saved.ip || '---',
        port: saved.port,
        fingerprint: saved.fingerprint || k,
        key: k
      };
    }
  });

  const f = (filterText || '').trim().toLowerCase();
  let contactKeys = Array.from(allKeys);
  if (f) {
    contactKeys = contactKeys.filter(k => {
      const p = peerMap[k];
      return (p.alias || '').toLowerCase().includes(f) || (p.ip || '').includes(f);
    });
  }

  contactKeys.sort((a, b) => {
    const convA = chatConversations[a] || chatConversations[peerMap[a]?.fingerprint?.toLowerCase()];
    const convB = chatConversations[b] || chatConversations[peerMap[b]?.fingerprint?.toLowerCase()];
    const timeA = convA?.lastTimestamp || 0;
    const timeB = convB?.lastTimestamp || 0;
    return timeB - timeA;
  });

  if (contactKeys.length === 0) {
    chatContactsList.innerHTML = `
      <div class="chat-empty-contacts">
        <p>Nenhum dispositivo encontrado na rede local.</p>
        <small style="color:var(--text-dim);">Abra o Ducker em outro computador ou o LocalSend no celular!</small>
      </div>
    `;
    return;
  }

  chatContactsList.innerHTML = '';
  contactKeys.forEach(k => {
    const info = peerMap[k];
    const conv = chatConversations[k] || chatConversations[info.fingerprint?.toLowerCase()] || { messages: [], unread: 0 };
    const lastMsg = conv.messages && conv.messages.length > 0 ? conv.messages[conv.messages.length - 1] : null;
    const isSelected = activeChatPeerKey && (
      activeChatPeerKey.toLowerCase() === k ||
      (info.fingerprint && activeChatPeerKey.toLowerCase() === info.fingerprint.toLowerCase())
    );
    const icon = getDeviceIcon(info.type, info.alias || info.model);

    let snippet = 'Nenhuma mensagem recente';
    let timeStr = '';
    if (lastMsg) {
      snippet = (lastMsg.direction === 'sent' ? 'Você: ' : '') + lastMsg.text;
      const d = new Date(lastMsg.timestamp);
      timeStr = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }

    const item = document.createElement('div');
    item.className = `chat-contact-item ${isSelected ? 'active' : ''}`;
    item.innerHTML = `
      <div class="chat-contact-avatar-box">
        <span>${icon}</span>
        <span class="chat-contact-dot ${info.online ? '' : 'offline'}" title="${info.online ? 'Online' : 'Offline'}"></span>
      </div>
      <div class="chat-contact-meta">
        <div class="chat-contact-row-top">
          <span class="chat-contact-name">${escapeHtml(info.alias)}</span>
          <span class="chat-contact-time">${timeStr}</span>
        </div>
        <div class="chat-contact-row-bottom">
          <span class="chat-contact-snippet">${escapeHtml(snippet)}</span>
          ${(conv.unread > 0 && !isSelected) ? `<span class="chat-contact-unread-badge">${conv.unread}</span>` : ''}
        </div>
      </div>
    `;
    item.addEventListener('click', () => {
      openChatWithPeer(info.key || k);
    });
    chatContactsList.appendChild(item);
  });
}

function openChatWithPeer(peerKey) {
  activeChatPeerKey = peerKey;
  navigateToPage('chat');

  const lowerKey = peerKey.toLowerCase();
  if (chatConversations[lowerKey]) {
    chatConversations[lowerKey].unread = 0;
    saveChatHistory();
  }
  updateChatBadges();

  if (chatEmptyState) chatEmptyState.style.display = 'none';
  if (chatActiveView) chatActiveView.style.display = 'flex';
  const layoutBox = document.querySelector('.chat-layout-box');
  if (layoutBox) layoutBox.classList.add('mobile-in-chat');

  const peer = currentPeers.find(p => {
    const k = p.key || p.info.fingerprint || `${p.ip}:${p.port}`;
    return k.toLowerCase() === lowerKey || p.info.fingerprint.toLowerCase() === lowerKey || p.info.alias.toLowerCase() === lowerKey;
  });

  const conv = chatConversations[lowerKey] || {};
  const alias = peer ? peer.info.alias : (conv.peerInfo?.alias || 'Dispositivo');
  const isOnline = !!peer;

  if (chatActiveName) chatActiveName.textContent = alias;
  if (chatActiveAvatar) chatActiveAvatar.textContent = getDeviceIcon(peer?.info?.deviceType || conv.peerInfo?.deviceType, alias);
  if (chatActiveSubtitle) {
    chatActiveSubtitle.innerHTML = isOnline
      ? `<span style="color:var(--status-online);">● Online via Wi-Fi</span> • ${peer.ip}`
      : `<span style="color:var(--text-dim);">○ Offline</span> • Visto anteriormente`;
  }

  renderChatMessages();
  renderChatContacts(chatSearchInput ? chatSearchInput.value : '');

  if (chatInputText) {
    chatInputText.focus();
  }
}

function renderChatMessages() {
  if (!chatMessagesStream || !activeChatPeerKey) return;

  const lowerKey = activeChatPeerKey.toLowerCase();
  const conv = chatConversations[lowerKey] || { messages: [] };

  if (!conv.messages || conv.messages.length === 0) {
    chatMessagesStream.innerHTML = `
      <div style="margin: auto; text-align: center; color: var(--text-dim); font-size: 0.85rem; padding: 20px;">
        <div style="font-size: 2.2rem; margin-bottom: 8px;">🦆💬</div>
        <p style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Nenhuma mensagem trocada ainda.</p>
        <small>Escreva abaixo para enviar seu primeiro recado via rede local!</small>
      </div>
    `;
    return;
  }

  chatMessagesStream.innerHTML = '';
  let lastDateStr = '';

  conv.messages.forEach(msg => {
    const d = new Date(msg.timestamp);
    const dateStr = d.toLocaleDateString([], { day: '2-digit', month: '2-digit', year: 'numeric' });
    if (dateStr !== lastDateStr) {
      lastDateStr = dateStr;
      const divider = document.createElement('div');
      divider.className = 'chat-date-divider';
      divider.textContent = isToday(d) ? 'Hoje' : (isYesterday(d) ? 'Ontem' : dateStr);
      chatMessagesStream.appendChild(divider);
    }

    const timeStr = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    const row = document.createElement('div');
    row.className = `chat-msg-row ${msg.direction}`;

    const formattedText = formatChatMessageText(msg.text);

    row.innerHTML = `
      <div class="chat-msg-bubble">
        <div class="chat-msg-text">${formattedText}</div>
        <div class="chat-msg-meta">
          <span>${timeStr}</span>
          ${msg.direction === 'sent' ? `<span class="chat-msg-status">${msg.status === 'sending' ? '⏳' : (msg.status === 'error' ? '⚠️' : '✓✓')}</span>` : ''}
        </div>
      </div>
    `;
    chatMessagesStream.appendChild(row);
  });

  chatMessagesStream.scrollTop = chatMessagesStream.scrollHeight;
}

async function sendChatMessage() {
  if (!chatInputText || !activeChatPeerKey) return;
  const text = chatInputText.value.trim();
  if (!text) return;

  chatInputText.value = '';
  chatInputText.style.height = 'auto';

  const lowerKey = activeChatPeerKey.toLowerCase();
  if (!chatConversations[lowerKey]) {
    const peer = currentPeers.find(p => (p.key || p.info.fingerprint || `${p.ip}:${p.port}`).toLowerCase() === lowerKey);
    chatConversations[lowerKey] = {
      peerInfo: {
        alias: peer ? peer.info.alias : 'Dispositivo',
        fingerprint: peer ? peer.info.fingerprint : lowerKey,
        deviceModel: peer ? peer.info.deviceModel : '',
        deviceType: peer ? peer.info.deviceType : 'desktop',
        ip: peer ? peer.ip : ''
      },
      unread: 0,
      lastTimestamp: Date.now(),
      messages: []
    };
  }

  const msgId = 'msg_' + Date.now() + '_' + Math.random().toString(36).substr(2, 5);
  const newMsg = {
    id: msgId,
    direction: 'sent',
    text,
    timestamp: Date.now(),
    status: 'sending'
  };

  chatConversations[lowerKey].messages.push(newMsg);
  chatConversations[lowerKey].lastTimestamp = Date.now();
  saveChatHistory();
  renderChatMessages();
  renderChatContacts(chatSearchInput ? chatSearchInput.value : '');
  playMessageChime(true);

  try {
    await invoke('send_text', {
      payload: { peer_key: activeChatPeerKey, text }
    });
    newMsg.status = 'sent';
    saveChatHistory();
    renderChatMessages();
  } catch (err) {
    console.error('Erro ao enviar mensagem:', err);
    newMsg.status = 'error';
    saveChatHistory();
    renderChatMessages();
    alert(`Falha ao enviar mensagem: ${err}`);
  }
}

async function handleChatFileAttach() {
  if (!activeChatPeerKey) {
    alert('Abra uma conversa primeiro para anexar arquivos.');
    return;
  }
  try {
    if (hasTauri) {
      const selected = await invoke('pick_files');
      if (selected && selected.length > 0) {
        const fileNames = selected.map(p => cleanDisplayName(p)).join(', ');
        
        const lowerKey = activeChatPeerKey.toLowerCase();
        const msgId = 'msg_' + Date.now() + '_' + Math.random().toString(36).substr(2, 5);
        const fileMsg = {
          id: msgId,
          direction: 'sent',
          text: `📎 Enviando: ${fileNames}...`,
          timestamp: Date.now(),
          status: 'sending'
        };

        if (!chatConversations[lowerKey]) {
          const peer = currentPeers.find(p => (p.key || p.info.fingerprint || `${p.ip}:${p.port}`).toLowerCase() === lowerKey);
          chatConversations[lowerKey] = {
            peerInfo: {
              alias: peer ? peer.info.alias : 'Dispositivo',
              fingerprint: peer ? peer.info.fingerprint : lowerKey,
              deviceModel: peer ? peer.info.deviceModel : '',
              deviceType: peer ? peer.info.deviceType : 'desktop',
              ip: peer ? peer.ip : ''
            },
            unread: 0,
            lastTimestamp: Date.now(),
            messages: []
          };
        }

        chatConversations[lowerKey].messages.push(fileMsg);
        chatConversations[lowerKey].lastTimestamp = Date.now();
        saveChatHistory();
        renderChatMessages();

        await invoke('send_files', {
          payload: { peer_key: activeChatPeerKey, paths: selected }
        });

        fileMsg.text = `📎 Arquivo(s) enviado(s): ${fileNames}`;
        fileMsg.status = 'sent';
        saveChatHistory();
        renderChatMessages();
        playMessageChime(true);
      }
    }
  } catch (err) {
    console.error('Falha ao enviar arquivo pelo chat:', err);
    alert(`Erro ao transferir arquivo: ${err}`);
  }
}

function clearCurrentChatHistory() {
  if (!activeChatPeerKey) return;
  const lowerKey = activeChatPeerKey.toLowerCase();
  const conv = chatConversations[lowerKey];
  const name = conv?.peerInfo?.alias || 'este dispositivo';
  if (confirm(`Tem certeza de que deseja limpar o histórico de mensagens com "${name}"?`)) {
    if (chatConversations[lowerKey]) {
      chatConversations[lowerKey].messages = [];
      chatConversations[lowerKey].lastTimestamp = Date.now();
      saveChatHistory();
      renderChatMessages();
      renderChatContacts(chatSearchInput ? chatSearchInput.value : '');
    }
  }
}

function handleIncomingTextMessage(sender, text) {
  const senderFp = (sender.fingerprint || sender.alias || 'unknown').toLowerCase();
  if (!chatConversations[senderFp]) {
    chatConversations[senderFp] = {
      peerInfo: {
        alias: sender.alias,
        fingerprint: sender.fingerprint,
        deviceModel: sender.deviceModel,
        deviceType: sender.deviceType,
        quac_id: sender.quacId
      },
      unread: 0,
      lastTimestamp: Date.now(),
      messages: []
    };
  } else {
    chatConversations[senderFp].peerInfo.alias = sender.alias;
  }

  const isCurrentChat = activeChatPeerKey && (
    activeChatPeerKey.toLowerCase() === senderFp ||
    activeChatPeerKey.toLowerCase() === sender.alias.toLowerCase()
  );

  const msgObj = {
    id: 'msg_' + Date.now() + '_' + Math.random().toString(36).substr(2, 5),
    direction: 'received',
    text,
    timestamp: Date.now()
  };

  chatConversations[senderFp].messages.push(msgObj);
  chatConversations[senderFp].lastTimestamp = Date.now();

  if (isCurrentChat) {
    chatConversations[senderFp].unread = 0;
  } else {
    chatConversations[senderFp].unread = (chatConversations[senderFp].unread || 0) + 1;
    showChatToast(sender.alias, text, senderFp);
  }

  saveChatHistory();
  playMessageChime(false);
  updateChatBadges();
  renderChatContacts(chatSearchInput ? chatSearchInput.value : '');

  if (isCurrentChat) {
    renderChatMessages();
  }
}

// Chat UI event bindings
chatSearchInput?.addEventListener('input', (e) => {
  renderChatContacts(e.target.value);
});

btnChatSend?.addEventListener('click', () => {
  sendChatMessage();
});

chatInputText?.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    sendChatMessage();
  }
});

chatInputText?.addEventListener('input', function() {
  this.style.height = 'auto';
  this.style.height = Math.min(this.scrollHeight, 120) + 'px';
});

btnChatBack?.addEventListener('click', () => {
  const layoutBox = document.querySelector('.chat-layout-box');
  if (layoutBox) layoutBox.classList.remove('mobile-in-chat');
  activeChatPeerKey = null;
  renderChatContacts(chatSearchInput ? chatSearchInput.value : '');
});

btnChatAttachFile?.addEventListener('click', () => {
  handleChatFileAttach();
});

btnChatSendFileDirect?.addEventListener('click', () => {
  handleChatFileAttach();
});

btnChatQuickDuck?.addEventListener('click', () => {
  if (!chatInputText) return;
  chatInputText.value += (chatInputText.value ? ' ' : '') + 'Quac! 🦆';
  chatInputText.focus();
});

btnChatClearHistory?.addEventListener('click', () => {
  clearCurrentChatHistory();
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
      handleIncomingTextMessage(ev.sender, ev.text);
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
      loadDuckive();
    } else if (ev.type === 'sessionFinished' || ev.type === 'sessionCancelled') {
      incomingModal.style.display = 'none';
      activeSessionId = null;
      loadDuckive();
    }
  });

  window.__TAURI__.event.listen('ducker://send-progress', (e) => {
    const p = e.payload;
    const pct = Math.min(100, Math.round((p.totalSent / p.total) * 100));
    btnExecuteSend.textContent = `Enviando: ${pct}%`;
  });

  window.__TAURI__.event.listen('ducker://duckive-unlock', (e) => {
    const summary = e.payload;
    console.log('[Duckive Unlock Event]', summary);
    showCelebrationModal(summary);
    loadDuckive();
  });

  window.__TAURI__.event.listen('ducker://duckive-updated', (e) => {
    const view = e.payload;
    updateDuckiveUI(view);
  });

  window.__TAURI__.event.listen('ducker://admin-changed', (e) => {
    currentAdminSession = e.payload;
    renderAdminUI();
  });

  window.__TAURI__.event.listen('ducker://update-download-progress', (e) => {
    const p = e.payload;
    handleUpdateDownloadProgress(p);
  });
}

// ==================== PAINEL DO ADMINISTRADOR (FAELINDO) ====================
let currentAdminSession = null;

async function checkAdminSession() {
  try {
    const session = await invoke('get_admin_session');
    currentAdminSession = session;
    renderAdminUI();
  } catch (err) {
    console.warn('[Admin] Erro ao verificar sessão admin:', err);
    currentAdminSession = null;
    renderAdminUI();
  }
}

function renderAdminUI() {
  const statusBadge = document.getElementById('adminStatusBadge');
  const loggedOutView = document.getElementById('adminLoggedOutView');
  const loggedInView = document.getElementById('adminLoggedInView');
  const sessionName = document.getElementById('adminSessionName');

  const isAdmin = currentAdminSession && currentAdminSession.isAdmin;

  if (statusBadge) {
    if (isAdmin) {
      statusBadge.textContent = '👑 Autenticado (Faelindo)';
      statusBadge.className = 'admin-status-badge status-logged-in';
    } else {
      statusBadge.textContent = 'Não Autenticado';
      statusBadge.className = 'admin-status-badge status-logged-out';
    }
  }

  if (loggedOutView) loggedOutView.style.display = isAdmin ? 'none' : 'flex';
  if (loggedInView) loggedInView.style.display = isAdmin ? 'flex' : 'none';
  if (sessionName && isAdmin) {
    sessionName.textContent = currentAdminSession.adminName || 'Faelindo';
  }
}

async function loginAdmin() {
  const input = document.getElementById('inputAdminCode');
  const code = (input?.value || '').trim();
  if (!code) {
    alert('Por favor, informe a chave de acesso de desenvolvedor.');
    return;
  }

  try {
    const session = await invoke('auth_admin', { code });
    currentAdminSession = session;
    if (input) input.value = '';
    renderAdminUI();
    alert('👑 Bem-vindo, Administrador Faelindo! Privilégios dev ativados com sucesso.');
  } catch (err) {
    alert(`Acesso negado: ${err}`);
  }
}

async function logoutAdmin() {
  try {
    await invoke('logout_admin');
    currentAdminSession = null;
    renderAdminUI();
    const modal = document.getElementById('adminModal');
    if (modal) modal.style.display = 'none';
    alert('Sessão de Administrador encerrada.');
  } catch (err) {
    console.error('Erro ao sair do modo admin:', err);
  }
}

async function openAdminModal() {
  const modal = document.getElementById('adminModal');
  if (!modal) return;
  modal.style.display = 'grid';
  await refreshAdminStats();
}

function closeAdminModal() {
  const modal = document.getElementById('adminModal');
  if (modal) modal.style.display = 'none';
}

async function refreshAdminStats() {
  try {
    const stats = await invoke('admin_get_stats');
    if (stats) {
      const elFilesSent = document.getElementById('admStatFilesSent');
      const elBytesSent = document.getElementById('admStatBytesSent');
      const elFilesRecv = document.getElementById('admStatFilesReceived');
      const elLvl = document.getElementById('admStatDuckiveLevel');
      const elTitle = document.getElementById('admStatDuckiveTitle');
      const elCosm = document.getElementById('admStatCosmeticsCount');
      const elPeers = document.getElementById('admStatPeersCount');
      const elPort = document.getElementById('admStatPort');
      const elSave = document.getElementById('admStatSaveDir');
      const elBuild = document.getElementById('admStatBuildMode');

      const filesSent = stats.total_files_sent ?? stats.totalFilesSent ?? 0;
      const bytesSent = stats.total_bytes_sent ?? stats.totalBytesSent ?? 0;
      const filesRecv = stats.total_files_received ?? stats.totalFilesReceived ?? 0;
      const lvl = stats.duckive_level ?? stats.duckiveLevel ?? 1;
      const title = stats.duckive_level_title ?? stats.duckiveLevelTitle ?? 'Pato Recruta';
      const itemsUnlocked = stats.items_unlocked_count ?? stats.itemsUnlockedCount ?? 0;
      const itemsTotal = stats.total_items_count ?? stats.totalItemsCount ?? 0;
      const peerCount = stats.peer_count ?? stats.peerCount ?? 0;
      const port = stats.port ?? 53317;
      const saveDir = stats.save_dir ?? stats.saveDir ?? '...';
      const isDebug = stats.is_debug ?? stats.isDebug ?? false;

      if (elFilesSent) elFilesSent.textContent = Number(filesSent).toLocaleString();
      if (elBytesSent) elBytesSent.textContent = `${(Number(bytesSent) / (1024 * 1024)).toFixed(1)} MB`;
      if (elFilesRecv) elFilesRecv.textContent = Number(filesRecv).toLocaleString();
      if (elLvl) elLvl.textContent = lvl;
      if (elTitle) elTitle.textContent = title;
      if (elCosm) elCosm.textContent = `${itemsUnlocked} / ${itemsTotal}`;
      if (elPeers) elPeers.textContent = peerCount;
      if (elPort) elPort.textContent = port;
      if (elSave) elSave.textContent = saveDir;
      if (elBuild) {
        elBuild.textContent = isDebug ? 'Debug Dev' : 'Release Prod';
        elBuild.style.background = isDebug ? 'rgba(16, 185, 129, 0.2)' : 'rgba(239, 68, 68, 0.2)';
        elBuild.style.color = isDebug ? '#10b981' : '#ef4444';
      }
    }

    renderAdminPeersList();
  } catch (err) {
    console.error('Falha ao obter estatísticas de administrador:', err);
  }
}

function renderAdminPeersList() {
  const container = document.getElementById('admPeersList');
  if (!container) return;

  if (currentPeers.length === 0) {
    container.innerHTML = `<div class="empty-hint">Nenhum dispositivo encontrado na rede no momento.</div>`;
    return;
  }

  container.innerHTML = currentPeers.map(p => `
    <div class="diag-peer-row">
      <div>
        <div class="diag-peer-alias">${escapeHtml(p.info.alias)} (${p.info.deviceType})</div>
        <div class="diag-peer-meta">${p.ip}:${p.port} • Protocolo: ${p.protocol} • Versão: ${p.info.version}</div>
      </div>
      <div class="mono" style="font-size: 0.72rem; color: var(--text-dim); max-width: 140px; overflow: hidden; text-overflow: ellipsis;">
        FP: ${p.info.fingerprint || 'N/A'}
      </div>
    </div>
  `).join('');
}

async function handleAdminSimulate(filesCount, bytes) {
  try {
    const summary = await invoke('admin_simulate_transfer', { filesCount, bytes });
    console.log('[Admin Simulation Result]', summary);
    await loadDuckive();
    await refreshAdminStats();
    if (summary && (summary.newlyUnlockedItems?.length > 0 || summary.levelUp)) {
      showCelebrationModal(summary);
    } else {
      alert(`Simulação concluída com sucesso! +${filesCount} arquivos adicionados.`);
    }
  } catch (err) {
    alert(`Erro na simulação: ${err}`);
  }
}

async function handleAdminTestUnlock() {
  const select = document.getElementById('selectTestCosmetic');
  const itemId = select?.value;
  if (!itemId) return;

  try {
    const view = await invoke('admin_test_unlock', { itemId });
    updateDuckiveUI(view);
    await refreshAdminStats();
    alert(`Item '${itemId}' desbloqueado com sucesso no inventário!`);
  } catch (err) {
    alert(`Erro ao desbloquear item: ${err}`);
  }
}

// Event listeners do Painel Admin
document.getElementById('btnAdminLogin')?.addEventListener('click', loginAdmin);
document.getElementById('inputAdminCode')?.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') loginAdmin();
});
document.getElementById('btnAdminLogout')?.addEventListener('click', logoutAdmin);
document.getElementById('btnOpenAdminModal')?.addEventListener('click', openAdminModal);
document.getElementById('btnCloseAdminModal')?.addEventListener('click', closeAdminModal);
document.getElementById('btnDoneAdmin')?.addEventListener('click', closeAdminModal);
document.getElementById('btnRefreshAdminData')?.addEventListener('click', refreshAdminStats);

document.getElementById('btnSimulate50k')?.addEventListener('click', () => {
  handleAdminSimulate(50000, 5000000000);
});
document.getElementById('btnSimulate100')?.addEventListener('click', () => {
  handleAdminSimulate(100, 10000000);
});
document.getElementById('btnSimulate1k')?.addEventListener('click', () => {
  handleAdminSimulate(1000, 100000000);
});
document.getElementById('btnUnlockSelectedCosmetic')?.addEventListener('click', handleAdminTestUnlock);

document.getElementById('btnAdmRescanNet')?.addEventListener('click', async () => {
  await scanSubnet();
  await refreshAdminStats();
});

// Abas do Modal Admin
document.querySelectorAll('.admin-modal-tabs .admin-tab-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.admin-modal-tabs .admin-tab-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    const tab = btn.dataset.admTab;
    document.querySelectorAll('.admin-modal-body .admin-tab-pane').forEach(p => p.style.display = 'none');
    if (tab === 'metrics') {
      const p = document.getElementById('admPaneMetrics');
      if (p) p.style.display = 'block';
    } else if (tab === 'simulation') {
      const p = document.getElementById('admPaneSimulation');
      if (p) p.style.display = 'block';
    } else if (tab === 'diagnostics') {
      const p = document.getElementById('admPaneDiagnostics');
      if (p) {
        p.style.display = 'block';
        renderAdminPeersList();
      }
    }
  });
});

// ==================== ATUALIZADOR DO DUCKER (GUI UPDATER) ====================
let currentUpdateInfo = null;
let downloadedInstallerPath = null;
let isDownloadingUpdate = false;

const btnCheckUpdates = document.getElementById('btnCheckUpdates');
const btnCancelUpdateDownload = document.getElementById('btnCancelUpdateDownload');
const btnInstallUpdate = document.getElementById('btnInstallUpdate');
const updaterStatusBox = document.getElementById('updaterStatusBox');
const updaterStatusDot = document.getElementById('updaterStatusDot');
const updaterStatusTitle = document.getElementById('updaterStatusTitle');
const updaterStatusMsg = document.getElementById('updaterStatusMsg');
const updaterProgressContainer = document.getElementById('updaterProgressContainer');
const updaterProgressBarFill = document.getElementById('updaterProgressBarFill');
const updaterProgressBytes = document.getElementById('updaterProgressBytes');
const updaterProgressPercent = document.getElementById('updaterProgressPercent');
const updaterNotesBox = document.getElementById('updaterNotesBox');
const updaterNotesContent = document.getElementById('updaterNotesContent');

function formatBytes(bytes) {
  if (!bytes || bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

function handleUpdateDownloadProgress(progress) {
  if (!updaterProgressContainer) return;
  updaterProgressContainer.style.display = 'block';
  if (updaterProgressBarFill) {
    updaterProgressBarFill.style.width = `${progress.pct}%`;
  }
  if (updaterProgressPercent) {
    updaterProgressPercent.textContent = `${progress.pct}%`;
  }
  if (updaterProgressBytes) {
    updaterProgressBytes.textContent = `${formatBytes(progress.downloaded)} / ${formatBytes(progress.total)}`;
  }
  if (updaterStatusTitle) {
    updaterStatusTitle.textContent = `Baixando atualização (${progress.pct}%)...`;
  }
}

async function checkForUpdates() {
  if (isDownloadingUpdate) return;

  if (updaterStatusBox) {
    updaterStatusBox.style.display = 'flex';
    updaterStatusBox.className = 'updater-status-box status-checking';
  }
  if (updaterStatusDot) {
    updaterStatusDot.className = 'updater-status-indicator dot-loading';
  }
  if (updaterStatusTitle) {
    updaterStatusTitle.textContent = 'Verificando atualizações...';
  }
  if (updaterStatusMsg) {
    updaterStatusMsg.textContent = 'Consultando o repositório oficial no GitHub (oFerrazDEV00/ducker)...';
  }
  if (updaterProgressContainer) {
    updaterProgressContainer.style.display = 'none';
  }
  if (updaterNotesBox) {
    updaterNotesBox.style.display = 'none';
  }
  if (btnInstallUpdate) {
    btnInstallUpdate.style.display = 'none';
  }
  if (btnCancelUpdateDownload) {
    btnCancelUpdateDownload.style.display = 'none';
  }

  if (btnCheckUpdates) {
    btnCheckUpdates.disabled = true;
    btnCheckUpdates.innerHTML = `<span>Buscando...</span>`;
  }

  try {
    const res = await invoke('check_for_updates');
    currentUpdateInfo = res;

    const currentBadge = document.getElementById('currentVersionBadge');
    if (currentBadge && res.currentVersion) {
      currentBadge.textContent = `v${res.currentVersion}`;
    }

    if (res.available && res.assetUrl) {
      if (updaterStatusBox) updaterStatusBox.className = 'updater-status-box status-available';
      if (updaterStatusDot) updaterStatusDot.className = 'updater-status-indicator dot-success';
      if (updaterStatusTitle) updaterStatusTitle.textContent = `Nova versão disponível: v${res.latestVersion}!`;
      if (updaterStatusMsg) updaterStatusMsg.textContent = res.statusMessage;

      if (res.releaseNotes && res.releaseNotes.trim().length > 0 && updaterNotesBox && updaterNotesContent) {
        updaterNotesContent.textContent = res.releaseNotes;
        updaterNotesBox.style.display = 'flex';
      }

      await startDownloadUpdate(res);
    } else {
      if (updaterStatusBox) updaterStatusBox.className = 'updater-status-box status-uptodate';
      if (updaterStatusDot) updaterStatusDot.className = 'updater-status-indicator dot-info';
      if (updaterStatusTitle) updaterStatusTitle.textContent = 'Ducker Atualizado';
      if (updaterStatusMsg) updaterStatusMsg.textContent = res.statusMessage;
    }
  } catch (err) {
    console.error('[Updater] Erro ao verificar atualizações:', err);
    if (updaterStatusBox) updaterStatusBox.className = 'updater-status-box status-error';
    if (updaterStatusDot) updaterStatusDot.className = 'updater-status-indicator dot-error';
    if (updaterStatusTitle) updaterStatusTitle.textContent = 'Falha na verificação';
    if (updaterStatusMsg) updaterStatusMsg.textContent = `Não foi possível consultar os servidores do GitHub: ${err}`;
  } finally {
    if (btnCheckUpdates) {
      btnCheckUpdates.disabled = false;
      btnCheckUpdates.innerHTML = `
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
        </svg>
        <span>Verificar atualizações</span>
      `;
    }
  }
}

async function startDownloadUpdate(info) {
  if (!info || !info.assetUrl || !info.assetName) return;

  isDownloadingUpdate = true;
  downloadedInstallerPath = null;

  if (btnCheckUpdates) btnCheckUpdates.style.display = 'none';
  if (btnCancelUpdateDownload) btnCancelUpdateDownload.style.display = 'inline-flex';
  if (btnInstallUpdate) btnInstallUpdate.style.display = 'none';

  if (updaterProgressContainer) {
    updaterProgressContainer.style.display = 'flex';
    if (updaterProgressBarFill) updaterProgressBarFill.style.width = '0%';
    if (updaterProgressPercent) updaterProgressPercent.textContent = '0%';
    if (updaterProgressBytes) updaterProgressBytes.textContent = `0 B / ${formatBytes(info.assetSize || 0)}`;
  }

  if (updaterStatusTitle) {
    updaterStatusTitle.textContent = `Baixando ${info.assetName}...`;
  }
  if (updaterStatusMsg) {
    updaterStatusMsg.textContent = 'Download seguro via HTTPS em andamento com validação SHA-256...';
  }

  try {
    const savedPath = await invoke('download_update', {
      assetUrl: info.assetUrl,
      assetName: info.assetName
    });

    downloadedInstallerPath = savedPath;
    isDownloadingUpdate = false;

    if (btnCancelUpdateDownload) btnCancelUpdateDownload.style.display = 'none';
    if (btnCheckUpdates) btnCheckUpdates.style.display = 'inline-flex';
    if (btnInstallUpdate) btnInstallUpdate.style.display = 'inline-flex';

    if (updaterStatusDot) updaterStatusDot.className = 'updater-status-indicator dot-success';
    if (updaterStatusTitle) updaterStatusTitle.textContent = 'Download concluído e validado!';
    if (updaterStatusMsg) {
      updaterStatusMsg.textContent = `O arquivo de atualização (${info.assetName}) está pronto para ser instalado. Clique no botão abaixo para prosseguir.`;
    }
  } catch (err) {
    isDownloadingUpdate = false;
    if (btnCancelUpdateDownload) btnCancelUpdateDownload.style.display = 'none';
    if (btnCheckUpdates) btnCheckUpdates.style.display = 'inline-flex';
    if (updaterProgressContainer) updaterProgressContainer.style.display = 'none';

    if (String(err).includes('cancelado')) {
      if (updaterStatusDot) updaterStatusDot.className = 'updater-status-indicator dot-info';
      if (updaterStatusTitle) updaterStatusTitle.textContent = 'Download cancelado';
      if (updaterStatusMsg) updaterStatusMsg.textContent = 'O download da atualização foi cancelado pelo usuário.';
    } else {
      if (updaterStatusDot) updaterStatusDot.className = 'updater-status-indicator dot-error';
      if (updaterStatusTitle) updaterStatusTitle.textContent = 'Falha no download';
      if (updaterStatusMsg) updaterStatusMsg.textContent = `Erro ao baixar arquivo: ${err}`;
    }
  }
}

async function cancelUpdateDownload() {
  try {
    await invoke('cancel_update_download');
  } catch (e) {
    console.warn('Erro ao solicitar cancelamento do download:', e);
  }
}

async function installUpdate() {
  if (!downloadedInstallerPath) {
    alert('Nenhum instalador baixado para executar.');
    return;
  }

  if (confirm('O instalador será iniciado para atualizar o Ducker. Deseja prosseguir agora?')) {
    try {
      await invoke('install_update', { filePath: downloadedInstallerPath });
      if (updaterStatusMsg) {
        updaterStatusMsg.textContent = 'Instalador iniciado com sucesso! Siga os passos na janela do instalador.';
      }
    } catch (e) {
      alert(`Falha ao iniciar o instalador: ${e}`);
    }
  }
}

btnCheckUpdates?.addEventListener('click', checkForUpdates);
btnCancelUpdateDownload?.addEventListener('click', cancelUpdateDownload);
btnInstallUpdate?.addEventListener('click', installUpdate);

// ==================== INICIALIZAÇÃO ====================
initTheme();
loadIdentity();
loadDuckive();
loadHistory();
checkAdminSession();
updateChatBadges();
renderChatContacts();
refreshPeers();
setInterval(refreshPeers, 8000);

// Auto-scan rápido após 2.5s se nenhum peer responder via multicast (essencial para redes Wi-Fi móveis)
setTimeout(() => {
  if (currentPeers.length === 0) {
    console.log('[Ducker Discovery] Executando busca ativa na sub-rede...');
    scanSubnet();
  }
}, 2500);


