import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  StyleSheet,
  Text,
  View,
  TextInput,
  TouchableOpacity,
  ScrollView,
  SafeAreaView,
  StatusBar,
  Modal,
  Platform,
  NativeModules,
  ActivityIndicator,
  Alert,
} from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import Constants from 'expo-constants';
import { File, Directory, Paths } from 'expo-file-system';
import * as Sharing from 'expo-sharing';
import * as DocumentPicker from 'expo-document-picker';

const STORAGE_KEY_IDENTITY = '@ducker_identity';
const STORAGE_KEY_NODE_IP = '@ducker_node_ip';
const DEFAULT_BRIDGE_PORT = 7876;
const RECONNECT_DELAY_MS = 2500;

function sanitizeFileName(name) {
  const cleaned = String(name || 'arquivo').replace(/[\\/:*?"<>|]/g, '_').trim();
  return cleaned || 'arquivo';
}

function uniqueFileIn(dir, fileName) {
  const safe = sanitizeFileName(fileName);
  let candidate = new File(dir, safe);
  if (!candidate.exists) return candidate;

  const dot = safe.lastIndexOf('.');
  const base = dot > 0 ? safe.slice(0, dot) : safe;
  const ext = dot > 0 ? safe.slice(dot) : '';
  for (let i = 1; i < 1000; i++) {
    candidate = new File(dir, `${base} (${i})${ext}`);
    if (!candidate.exists) return candidate;
  }
  return new File(dir, `${base}-${Date.now()}${ext}`);
}

function formatSize(bytes) {
  if (!bytes) return '0 KB';
  if (bytes >= 1024 * 1024) return (bytes / 1024 / 1024).toFixed(2) + ' MB';
  return (bytes / 1024).toFixed(1) + ' KB';
}

// Descobre o IP do PC automaticamente a partir do ambiente do Expo / Metro
function getInferredHostIp() {
  try {
    const scriptURL = NativeModules.SourceCode?.scriptURL;
    if (scriptURL) {
      const match = scriptURL.match(/https?:\/\/([^/:]+)/);
      if (match && match[1] && match[1] !== 'localhost' && match[1] !== '127.0.0.1') {
        return match[1];
      }
    }

    const hostUri =
      Constants?.expoConfig?.hostUri ||
      Constants?.manifest2?.extra?.expoGo?.debuggerHost ||
      Constants?.manifest?.debuggerHost;
    if (hostUri) {
      const ip = hostUri.split(':')[0];
      if (ip && ip !== 'localhost' && ip !== '127.0.0.1') {
        return ip;
      }
    }
  } catch (e) {}
  return null;
}

export default function App() {
  const [activeTab, setActiveTab] = useState('receive'); // 'receive' | 'send'
  const [identity, setIdentity] = useState(null);
  const [deviceNameInput, setDeviceNameInput] = useState('');
  const [isConfiguring, setIsConfiguring] = useState(false);

  // Conexão com PC/Node
  const [nodeIp, setNodeIp] = useState('');
  const [nodeIpInput, setNodeIpInput] = useState('');
  const [showManualIp, setShowManualIp] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [connectedNode, setConnectedNode] = useState(null);
  const [connectedPcQuacId, setConnectedPcQuacId] = useState(null);
  const [connectionStatus, setConnectionStatus] = useState('Buscando PC...');
  const wsRef = useRef(null);
  const identityRef = useRef(null);
  const nodeIpRef = useRef('');

  // Transferências recebidas e enviadas
  const [receivedTransfers, setReceivedTransfers] = useState([]);
  const [sentTransfers, setSentTransfers] = useState([]);

  // Estado de envio (iPhone -> PC)
  const [messageText, setMessageText] = useState('');
  const [isSendingMessage, setIsSendingMessage] = useState(false);
  const [selectedFile, setSelectedFile] = useState(null);
  const [isUploadingFile, setIsUploadingFile] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);

  // Modal de transferência ativa (Recebimento)
  const [activeTransfer, setActiveTransfer] = useState(null);
  const activeTransferIdRef = useRef(null);
  const [transferProgress, setTransferProgress] = useState(0);
  const [transferError, setTransferError] = useState(null);
  const [transferSuccess, setTransferSuccess] = useState(false);
  const [savedFileUri, setSavedFileUri] = useState(null);

  identityRef.current = identity;
  nodeIpRef.current = nodeIp;

  // Carregar identidade e IP salvo ou inferido
  useEffect(() => {
    async function loadStored() {
      try {
        const [storedIdentity, storedIp] = await Promise.all([
          AsyncStorage.getItem(STORAGE_KEY_IDENTITY),
          AsyncStorage.getItem(STORAGE_KEY_NODE_IP),
        ]);

        const inferred = getInferredHostIp();
        const initialIp = storedIp || inferred || '';

        if (initialIp) {
          setNodeIp(initialIp);
          setNodeIpInput(initialIp);
        }

        if (storedIdentity) {
          setIdentity(JSON.parse(storedIdentity));
        } else {
          setIsConfiguring(true);
        }
      } catch (e) {
        setIsConfiguring(true);
      }
    }
    loadStored();
  }, []);

  // Busca dados de status do PC
  const fetchPcStatus = useCallback(async (ip) => {
    if (!ip) return;
    try {
      const res = await fetch(`http://${ip}:${DEFAULT_BRIDGE_PORT}/api/status`);
      if (res.ok) {
        const data = await res.json();
        setConnectedNode(data.device_name);
        setConnectedPcQuacId(data.quac_id);
      }
    } catch (e) {}
  }, []);

  // Varredura da rede local
  const scanForDucker = useCallback(async (baseIp) => {
    if (!baseIp) return null;
    const parts = baseIp.split('.');
    if (parts.length !== 4) return null;
    const subnet = `${parts[0]}.${parts[1]}.${parts[2]}`;

    setIsScanning(true);
    setConnectionStatus('Procurando PC na rede...');

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 800);
      const res = await fetch(`http://${baseIp}:${DEFAULT_BRIDGE_PORT}/api/status`, {
        signal: controller.signal,
      });
      clearTimeout(timeoutId);
      if (res.ok) {
        setIsScanning(false);
        return baseIp;
      }
    } catch (e) {}

    const candidates = [];
    for (let i = 1; i <= 254; i++) {
      candidates.push(`${subnet}.${i}`);
    }

    const batchSize = 15;
    for (let i = 0; i < candidates.length; i += batchSize) {
      const batch = candidates.slice(i, i + batchSize);
      const promises = batch.map(async (ip) => {
        try {
          const controller = new AbortController();
          const timeoutId = setTimeout(() => controller.abort(), 600);
          const res = await fetch(`http://${ip}:${DEFAULT_BRIDGE_PORT}/api/status`, {
            signal: controller.signal,
          });
          clearTimeout(timeoutId);
          if (res.ok) {
            return ip;
          }
        } catch (e) {
          return null;
        }
        return null;
      });

      const results = await Promise.all(promises);
      const found = results.find((ip) => ip !== null);
      if (found) {
        setIsScanning(false);
        return found;
      }
    }

    setIsScanning(false);
    return null;
  }, []);

  // Conectar WebSocket ao PC
  useEffect(() => {
    if (!identity) return;

    let cancelled = false;
    let socket = null;
    let retryTimer = null;

    async function attemptConnection() {
      if (cancelled) return;

      let currentIp = nodeIpRef.current;
      if (!currentIp) {
        currentIp = getInferredHostIp();
        if (currentIp) {
          setNodeIp(currentIp);
          setNodeIpInput(currentIp);
        }
      }

      if (!currentIp) {
        setConnectionStatus('Aguardando IP...');
        retryTimer = setTimeout(attemptConnection, RECONNECT_DELAY_MS);
        return;
      }

      setConnectionStatus('Conectando ao PC...');
      try {
        socket = new WebSocket(`ws://${currentIp}:${DEFAULT_BRIDGE_PORT}/api/ws`);
      } catch (e) {
        setConnectionStatus('Erro ao abrir socket');
        retryTimer = setTimeout(attemptConnection, RECONNECT_DELAY_MS);
        return;
      }
      wsRef.current = socket;

      socket.onopen = () => {
        setConnectionStatus('Conectado');
        fetchPcStatus(currentIp);
        socket.send(
          JSON.stringify({
            type: 'mobile_announce',
            quac_id: Number(identity.quac_id),
            device_name: identity.device_name,
          })
        );
      };

      socket.onmessage = (event) => {
        try {
          handleIncomingMessage(JSON.parse(event.data));
        } catch (err) {
          console.error('Erro ao processar mensagem WS:', err);
        }
      };

      socket.onerror = () => {};

      socket.onclose = () => {
        setConnectedNode(null);
        setConnectedPcQuacId(null);
        if (cancelled) return;
        setConnectionStatus('Aguardando PC...');
        retryTimer = setTimeout(attemptConnection, RECONNECT_DELAY_MS);
      };
    }

    attemptConnection();

    return () => {
      cancelled = true;
      if (retryTimer) clearTimeout(retryTimer);
      if (socket) socket.close();
      wsRef.current = null;
    };
  }, [identity, nodeIp, fetchPcStatus]);

  function sendWs(obj) {
    const ws = wsRef.current;
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(obj));
      return true;
    }
    return false;
  }

  function addReceivedHistory(entry) {
    setReceivedTransfers((prev) => [
      { id: `${Date.now()}-${Math.random()}`, date: new Date().toLocaleTimeString(), ...entry },
      ...prev,
    ]);
  }

  function addSentHistory(entry) {
    setSentTransfers((prev) => [
      { id: `${Date.now()}-${Math.random()}`, date: new Date().toLocaleTimeString(), ...entry },
      ...prev,
    ]);
  }

  // Tratar mensagens recebidas do PC
  function handleIncomingMessage(msg) {
    if (msg.type === 'transfer_status' && msg.status === 'CONNECTED') {
      setConnectedNode(msg.message);
      if (nodeIpRef.current) fetchPcStatus(nodeIpRef.current);
    } else if (msg.type === 'transfer_progress') {
      if (msg.transfer_id === activeTransferIdRef.current && msg.total_bytes > 0) {
        setTransferProgress(Math.min(99, Math.floor((msg.bytes_sent / msg.total_bytes) * 100)));
      }
    } else if (msg.type === 'transfer_request') {
      handleTransferRequest(msg);
    }
  }

  function handleTransferRequest(req) {
    const own = identityRef.current;
    if (!own) return;

    activeTransferIdRef.current = req.transfer_id || null;
    setActiveTransfer(req);
    setTransferProgress(0);
    setTransferError(null);
    setTransferSuccess(false);
    setSavedFileUri(null);

    const isMatch = Number(req.destination_quac_id) === Number(own.quac_id);

    if (!isMatch) {
      const errorMsg =
        'Ocorreu um erro: a transferência foi interceptada porque o ID Quac de destino não corresponde a este dispositivo.';
      setTransferError({ code: 'DESTINATION_ID_MISMATCH', message: errorMsg });

      if (!req.simulated) {
        sendWs({
          type: 'handshake_response',
          transfer_id: req.transfer_id,
          status: 'REJECTED',
          reason: 'DESTINATION_ID_MISMATCH',
          message: errorMsg,
        });
      }

      addReceivedHistory({
        fileName: req.file_name,
        sender: req.sender_name,
        senderQuac: req.sender_quac_id,
        status: 'INTERCEPTADO',
        error: errorMsg,
      });
      return;
    }

    if (req.simulated) {
      simulateDownload(req);
      return;
    }

    sendWs({
      type: 'handshake_response',
      transfer_id: req.transfer_id,
      status: 'ACCEPTED',
      reason: null,
      message: null,
    });
    downloadTransfer(req);
  }

  async function downloadTransfer(req) {
    const url = `http://${nodeIpRef.current}:${DEFAULT_BRIDGE_PORT}${req.download_path}`;
    try {
      const dir = new Directory(Paths.document, 'Ducker');
      if (!dir.exists) dir.create();

      const target = uniqueFileIn(dir, req.file_name);
      const saved = await File.downloadFileAsync(url, target);

      if (typeof req.file_size === 'number' && saved.size !== req.file_size) {
        throw new Error(`Tamanho divergente: esperado ${req.file_size}, recebido ${saved.size}`);
      }

      sendWs({ type: 'transfer_complete', transfer_id: req.transfer_id, success: true, message: null });

      setTransferProgress(100);
      setTransferSuccess(true);
      setSavedFileUri(saved.uri);
      addReceivedHistory({
        fileName: req.file_name,
        sender: req.sender_name,
        senderQuac: req.sender_quac_id,
        status: 'RECEBIDO',
        size: formatSize(req.file_size || saved.size || 0),
        uri: saved.uri,
      });
    } catch (e) {
      const message = `Falha ao baixar/salvar: ${e?.message || e}`;
      sendWs({ type: 'transfer_complete', transfer_id: req.transfer_id, success: false, message });
      setTransferError({ code: 'TRANSFER_FAILED', message });
      addReceivedHistory({
        fileName: req.file_name,
        sender: req.sender_name,
        senderQuac: req.sender_quac_id,
        status: 'FALHOU',
        error: message,
      });
    }
  }

  function simulateDownload(req) {
    let current = 0;
    const interval = setInterval(() => {
      current += 25;
      if (current >= 100) {
        clearInterval(interval);
        setTransferProgress(100);
        setTransferSuccess(true);
        addReceivedHistory({
          fileName: req.file_name,
          sender: req.sender_name,
          senderQuac: req.sender_quac_id,
          status: 'SIMULADO',
          size: formatSize(req.file_size),
        });
      } else {
        setTransferProgress(current);
      }
    }, 150);
  }

  async function shareFile(uri) {
    try {
      if (await Sharing.isAvailableAsync()) {
        await Sharing.shareAsync(uri);
      }
    } catch (e) {
      console.error('Falha ao compartilhar:', e);
    }
  }

  // ================= ENVIAR DO IPHONE PARA O PC =================

  // 1. Enviar Mensagem Curta
  async function handleSendMessageToPc() {
    const trimmed = messageText.trim();
    if (!trimmed) {
      Alert.alert('Aviso', 'Digite uma mensagem para enviar.');
      return;
    }
    if (!nodeIpRef.current) {
      Alert.alert('Erro', 'Nenhum PC conectado na rede local.');
      return;
    }

    setIsSendingMessage(true);
    try {
      const destQuac = connectedPcQuacId || 0;
      const res = await fetch(`http://${nodeIpRef.current}:${DEFAULT_BRIDGE_PORT}/api/message`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sender_name: identity.device_name,
          sender_quac_id: Number(identity.quac_id),
          destination_quac_id: Number(destQuac),
          text: trimmed,
        }),
      });

      const data = await res.json();
      if (res.ok && data.success) {
        setMessageText('');
        addSentHistory({
          title: `Mensagem: "${trimmed.slice(0, 30)}${trimmed.length > 30 ? '...' : ''}"`,
          type: 'MENSAGEM',
          status: 'ENTREGUE',
        });
        Alert.alert('Sucesso! 🦆', 'Mensagem entregue e salva no PC!');
      } else {
        throw new Error(data.message || 'Falha ao enviar');
      }
    } catch (e) {
      Alert.alert('Erro no Envio', String(e?.message || e));
    } finally {
      setIsSendingMessage(false);
    }
  }

  // 2. Escolher Arquivo / Foto do iPhone
  async function handlePickDocument() {
    try {
      const result = await DocumentPicker.getDocumentAsync({
        copyToCacheDirectory: true,
        type: '*/*',
      });

      if (!result.canceled && result.assets && result.assets.length > 0) {
        const file = result.assets[0];
        setSelectedFile(file);
      }
    } catch (e) {
      Alert.alert('Erro ao selecionar arquivo', String(e?.message || e));
    }
  }

  // 3. Enviar Arquivo Selecionado para o PC
  async function handleUploadFileToPc() {
    if (!selectedFile) {
      Alert.alert('Aviso', 'Escolha um arquivo primeiro.');
      return;
    }
    if (!nodeIpRef.current) {
      Alert.alert('Erro', 'Nenhum PC conectado na rede local.');
      return;
    }

    setIsUploadingFile(true);
    setUploadProgress(0);

    try {
      const destQuac = connectedPcQuacId || 0;
      const safeName = sanitizeFileName(selectedFile.name);
      const url = `http://${nodeIpRef.current}:${DEFAULT_BRIDGE_PORT}/api/upload`;

      // Upload do arquivo
      const fileToUpload = new File(selectedFile.uri);
      const task = fileToUpload.createUploadTask(url, {
        headers: {
          'x-sender-name': identity.device_name,
          'x-sender-quac-id': String(identity.quac_id),
          'x-destination-quac-id': String(destQuac),
          'x-file-name': encodeURIComponent(safeName),
        },
      });

      const response = await task.uploadAsync();

      if (response && response.status >= 200 && response.status < 300) {
        addSentHistory({
          title: safeName,
          type: 'ARQUIVO',
          status: 'ENTREGUE',
          size: formatSize(selectedFile.size),
        });
        setSelectedFile(null);
        setUploadProgress(100);
        Alert.alert('Sucesso! 🦆', `Arquivo "${safeName}" enviado e salvo no PC com sucesso!`);
      } else {
        const errorText = response?.body || 'Falha na resposta do servidor.';
        throw new Error(errorText);
      }
    } catch (e) {
      Alert.alert('Erro no Upload', String(e?.message || e));
    } finally {
      setIsUploadingFile(false);
      setUploadProgress(0);
    }
  }

  async function handleManualConnect() {
    const ip = nodeIpInput.trim();
    if (!ip) return;
    try {
      await AsyncStorage.setItem(STORAGE_KEY_NODE_IP, ip);
    } catch (e) {}
    setNodeIp(ip);
  }

  async function handleAutoDetect() {
    const base = nodeIp || getInferredHostIp() || '192.168.18.1';
    const found = await scanForDucker(base);
    if (found) {
      try {
        await AsyncStorage.setItem(STORAGE_KEY_NODE_IP, found);
      } catch (e) {}
      setNodeIp(found);
      setNodeIpInput(found);
    }
  }

  async function handleSaveIdentity() {
    const trimmed = deviceNameInput.trim();
    if (!trimmed) return;

    const generatedQuacId = Math.floor(10000000 + Math.random() * 90000000);
    const newIdentity = {
      device_name: trimmed,
      quac_id: generatedQuacId,
    };

    try {
      await AsyncStorage.setItem(STORAGE_KEY_IDENTITY, JSON.stringify(newIdentity));
      setIdentity(newIdentity);
      setIsConfiguring(false);
    } catch (e) {
      console.error('Falha ao salvar identidade:', e);
    }
  }

  function testTransferValidation(simulateMismatch = false) {
    if (!identity) return;

    handleTransferRequest({
      type: 'transfer_request',
      simulated: true,
      transfer_id: `sim_${Date.now()}`,
      protocol_version: 1,
      sender_name: 'Notebook do Fael',
      sender_quac_id: 84726193,
      destination_quac_id: simulateMismatch ? 99999999 : identity.quac_id,
      file_name: simulateMismatch ? 'documento_confidencial.pdf' : 'foto_fael.png',
      file_size: 204800,
    });
  }

  function closeModal() {
    activeTransferIdRef.current = null;
    setActiveTransfer(null);
  }

  if (isConfiguring || !identity) {
    return (
      <SafeAreaView style={styles.container}>
        <StatusBar barStyle="light-content" />
        <View style={styles.wizardContent}>
          <Text style={styles.logo}>🦆</Text>
          <Text style={styles.title}>Bem-vindo ao Ducker</Text>
          <Text style={styles.subtitle}>Be simple, be duck.</Text>

          <View style={styles.card}>
            <Text style={styles.label}>Nome do dispositivo:</Text>
            <TextInput
              style={styles.input}
              placeholder="Ex: iPhone do Gabriel"
              placeholderTextColor="#64748b"
              value={deviceNameInput}
              onChangeText={setDeviceNameInput}
              autoFocus
            />

            <TouchableOpacity style={styles.buttonPrimary} onPress={handleSaveIdentity}>
              <Text style={styles.buttonPrimaryText}>Gerar meu ID Quac 🦆</Text>
            </TouchableOpacity>
          </View>
        </View>
      </SafeAreaView>
    );
  }

  const isConnected = connectionStatus === 'Conectado';
  const isDownloading = activeTransfer && !transferError && !transferSuccess;

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" />
      <ScrollView contentContainerStyle={styles.scrollContent} keyboardShouldPersistTaps="handled">
        {/* CABEÇALHO */}
        <View style={styles.header}>
          <View style={styles.headerTitleRow}>
            <Text style={styles.duckEmoji}>🦆</Text>
            <View>
              <Text style={styles.brandTitle}>DUCKER MOBILE</Text>
              <Text style={styles.brandTagline}>Be simple, be duck.</Text>
            </View>
          </View>
        </View>

        {/* CARTÃO DE IDENTIDADE DO IPHONE */}
        <View style={styles.identityCard}>
          <View style={styles.identityHeader}>
            <Text style={styles.identityLabel}>DISPOSITIVO LOCAL</Text>
            <View style={isConnected ? styles.badgeActive : styles.badgeIdle}>
              <View style={isConnected ? styles.dotOnline : styles.dotIdle} />
              <Text style={isConnected ? styles.badgeText : styles.badgeIdleText}>
                {isConnected ? 'Conectado ao PC' : 'Pronto para receber'}
              </Text>
            </View>
          </View>

          <Text style={styles.deviceName}>{identity.device_name}</Text>

          <View style={styles.quacBox}>
            <Text style={styles.quacLabel}>Seu ID Quac:</Text>
            <Text style={styles.quacId}>{identity.quac_id}</Text>
          </View>
        </View>

        {/* STATUS DA CONEXÃO AUTOMÁTICA */}
        <View style={styles.sectionCard}>
          <View style={styles.sectionTitleRow}>
            <Text style={styles.sectionTitle}>Conexão com PC</Text>
            {isScanning && <ActivityIndicator size="small" color="#f59e0b" />}
          </View>

          <View
            style={[
              styles.statusBanner,
              isConnected ? styles.statusBannerGreen : styles.statusBannerNeutral,
            ]}
          >
            <View style={isConnected ? styles.dotLargeOnline : styles.dotLargePulse} />
            <View style={{ flex: 1 }}>
              <Text style={styles.statusBannerTitle}>
                {isConnected
                  ? `${connectedNode || 'PC Conectado'} ${connectedPcQuacId ? `[Quac: ${connectedPcQuacId}]` : ''}`
                  : 'Aguardando PC na rede...'}
              </Text>
              <Text style={styles.statusBannerSub}>
                {isConnected
                  ? `IP: ${nodeIp}:${DEFAULT_BRIDGE_PORT}`
                  : `Monitorando ${nodeIp ? nodeIp : 'sua rede Wi-Fi'}`}
              </Text>
            </View>
          </View>

          <View style={styles.autoActionRow}>
            <TouchableOpacity
              style={styles.buttonSecondary}
              onPress={handleAutoDetect}
              disabled={isScanning}
            >
              <Text style={styles.buttonSecondaryText}>
                {isScanning ? '🔍 Varrendo rede...' : '🔍 Buscar PC'}
              </Text>
            </TouchableOpacity>

            <TouchableOpacity
              style={styles.buttonLink}
              onPress={() => setShowManualIp((prev) => !prev)}
            >
              <Text style={styles.buttonLinkText}>
                {showManualIp ? 'Ocultar IP' : 'Ajustar IP'}
              </Text>
            </TouchableOpacity>
          </View>

          {showManualIp && (
            <View style={styles.manualIpBox}>
              <Text style={styles.manualIpLabel}>IP específico do PC:</Text>
              <View style={styles.row}>
                <TextInput
                  style={[styles.input, { flex: 1, marginRight: 8, marginBottom: 0 }]}
                  value={nodeIpInput}
                  onChangeText={setNodeIpInput}
                  placeholder="Ex: 192.168.18.8"
                  placeholderTextColor="#64748b"
                  keyboardType={Platform.OS === 'ios' ? 'numbers-and-punctuation' : 'decimal-pad'}
                  autoCapitalize="none"
                  autoCorrect={false}
                />
                <TouchableOpacity style={styles.buttonConnect} onPress={handleManualConnect}>
                  <Text style={styles.buttonConnectText}>Salvar</Text>
                </TouchableOpacity>
              </View>
            </View>
          )}
        </View>

        {/* NAVEGAÇÃO DE ABAS: RECEBER / ENVIAR */}
        <View style={styles.tabContainer}>
          <TouchableOpacity
            style={[styles.tabButton, activeTab === 'receive' && styles.tabButtonActive]}
            onPress={() => setActiveTab('receive')}
          >
            <Text
              style={[styles.tabButtonText, activeTab === 'receive' && styles.tabButtonTextActive]}
            >
              📥 Receber do PC
            </Text>
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.tabButton, activeTab === 'send' && styles.tabButtonActive]}
            onPress={() => setActiveTab('send')}
          >
            <Text style={[styles.tabButtonText, activeTab === 'send' && styles.tabButtonTextActive]}>
              📤 Enviar para o PC
            </Text>
          </TouchableOpacity>
        </View>

        {/* ================= ABA ENVIAR ================= */}
        {activeTab === 'send' && (
          <View>
            {/* ENVIAR MENSAGEM */}
            <View style={styles.sectionCard}>
              <Text style={styles.sectionTitle}>✉️ Enviar Mensagem Curta</Text>
              <Text style={styles.sectionSubtitle}>
                A mensagem aparecerá na tela do PC e será salva como arquivo de texto:
              </Text>

              <TextInput
                style={styles.messageInput}
                placeholder="Ex: Olá do meu iPhone! 🦆"
                placeholderTextColor="#64748b"
                value={messageText}
                onChangeText={setMessageText}
                multiline
                numberOfLines={3}
              />

              <TouchableOpacity
                style={[
                  styles.buttonPrimary,
                  (!isConnected || isSendingMessage) && { opacity: 0.6 },
                ]}
                onPress={handleSendMessageToPc}
                disabled={!isConnected || isSendingMessage}
              >
                {isSendingMessage ? (
                  <ActivityIndicator color="#0f172a" />
                ) : (
                  <Text style={styles.buttonPrimaryText}>Enviar Mensagem para o PC 🚀</Text>
                )}
              </TouchableOpacity>
            </View>

            {/* ENVIAR ARQUIVO OU FOTO */}
            <View style={styles.sectionCard}>
              <Text style={styles.sectionTitle}>📁 Enviar Arquivo ou Foto</Text>
              <Text style={styles.sectionSubtitle}>
                Escolha qualquer documento, foto, PDF ou vídeo do iPhone:
              </Text>

              <TouchableOpacity style={styles.pickFileButton} onPress={handlePickDocument}>
                <Text style={styles.pickFileButtonText}>
                  {selectedFile ? '🔄 Escolher outro arquivo' : '📂 Selecionar do iPhone'}
                </Text>
              </TouchableOpacity>

              {selectedFile && (
                <View style={styles.selectedFileCard}>
                  <Text style={styles.selectedFileName} numberOfLines={1}>
                    📄 {selectedFile.name}
                  </Text>
                  <Text style={styles.selectedFileSize}>
                    Tamanho: {formatSize(selectedFile.size)}
                  </Text>

                  {isUploadingFile && (
                    <View style={{ marginTop: 10 }}>
                      <ActivityIndicator color="#f59e0b" />
                      <Text style={styles.uploadingText}>Enviando para o PC...</Text>
                    </View>
                  )}

                  <TouchableOpacity
                    style={[
                      styles.buttonPrimary,
                      { marginTop: 12 },
                      (!isConnected || isUploadingFile) && { opacity: 0.6 },
                    ]}
                    onPress={handleUploadFileToPc}
                    disabled={!isConnected || isUploadingFile}
                  >
                    <Text style={styles.buttonPrimaryText}>
                      {isUploadingFile ? 'Enviando...' : 'Enviar Arquivo para o PC 🚀'}
                    </Text>
                  </TouchableOpacity>
                </View>
              )}
            </View>

            {/* HISTÓRICO DE ENVIOS */}
            <View style={styles.sectionCard}>
              <Text style={styles.sectionTitle}>Histórico de Envios</Text>
              {sentTransfers.length === 0 ? (
                <Text style={styles.emptyText}>Nenhum envio realizado nesta sessão.</Text>
              ) : (
                sentTransfers.map((item) => (
                  <View key={item.id} style={[styles.transferItem, styles.itemAccepted]}>
                    <View style={styles.transferItemHeader}>
                      <Text style={styles.transferFileName} numberOfLines={1}>
                        📤 {item.title}
                      </Text>
                      <Text style={[styles.transferStatusBadge, styles.badgeGreen]}>
                        {item.status}
                      </Text>
                    </View>
                    <Text style={styles.transferSender}>
                      {item.type} • {item.date} {item.size ? `• ${item.size}` : ''}
                    </Text>
                  </View>
                ))
              )}
            </View>
          </View>
        )}

        {/* ================= ABA RECEBER ================= */}
        {activeTab === 'receive' && (
          <View>
            {/* SIMULAÇÃO DE TESTES DO P0 */}
            <View style={styles.sectionCard}>
              <Text style={styles.sectionTitle}>Testes Rápidos do Protocolo P0</Text>
              <Text style={styles.sectionSubtitle}>
                Valide a interceptação de ID Quac divergente (sem afetar o PC):
              </Text>

              <View style={styles.buttonRow}>
                <TouchableOpacity
                  style={[styles.testButton, styles.testButtonGreen]}
                  onPress={() => testTransferValidation(false)}
                >
                  <Text style={styles.testButtonText}>✓ Teste ID Correto</Text>
                </TouchableOpacity>

                <TouchableOpacity
                  style={[styles.testButton, styles.testButtonRed]}
                  onPress={() => testTransferValidation(true)}
                >
                  <Text style={styles.testButtonText}>✗ Teste ID Errado</Text>
                </TouchableOpacity>
              </View>
            </View>

            {/* HISTÓRICO DE RECEBIDOS */}
            <View style={styles.sectionCard}>
              <Text style={styles.sectionTitle}>Transferências Recebidas</Text>

              {receivedTransfers.length === 0 ? (
                <Text style={styles.emptyText}>Nenhum arquivo recebido ainda.</Text>
              ) : (
                receivedTransfers.map((item) => {
                  const failed = item.status === 'INTERCEPTADO' || item.status === 'FALHOU';
                  return (
                    <View
                      key={item.id}
                      style={[
                        styles.transferItem,
                        failed ? styles.itemRejected : styles.itemAccepted,
                      ]}
                    >
                      <View style={styles.transferItemHeader}>
                        <Text style={styles.transferFileName} numberOfLines={1}>
                          📦 {item.fileName}
                        </Text>
                        <Text
                          style={[
                            styles.transferStatusBadge,
                            failed ? styles.badgeRed : styles.badgeGreen,
                          ]}
                        >
                          {item.status}
                        </Text>
                      </View>
                      <Text style={styles.transferSender}>
                        De: {item.sender} [Quac: {item.senderQuac}] • {item.date}
                        {item.size ? ` • ${item.size}` : ''}
                      </Text>
                      {item.error && <Text style={styles.transferErrorText}>{item.error}</Text>}
                      {item.uri && (
                        <TouchableOpacity
                          style={styles.shareButtonSmall}
                          onPress={() => shareFile(item.uri)}
                        >
                          <Text style={styles.shareButtonText}>Salvar / Compartilhar</Text>
                        </TouchableOpacity>
                      )}
                    </View>
                  );
                })
              )}
            </View>
          </View>
        )}
      </ScrollView>

      {/* MODAL DE TRANSFERÊNCIA ATIVA */}
      <Modal visible={!!activeTransfer} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalCard}>
            <Text style={styles.modalTitle}>
              {transferError
                ? transferError.code === 'DESTINATION_ID_MISMATCH'
                  ? 'Transferência Rejeitada'
                  : 'Falha na Transferência'
                : 'Recebendo Arquivo'}
            </Text>

            {activeTransfer && (
              <View style={styles.modalBody}>
                <Text style={styles.modalFileName}>{activeTransfer.file_name}</Text>
                <Text style={styles.modalSender}>
                  De: {activeTransfer.sender_name} [Quac: {activeTransfer.sender_quac_id}]
                  {typeof activeTransfer.file_size === 'number'
                    ? ` • ${formatSize(activeTransfer.file_size)}`
                    : ''}
                </Text>

                <View style={styles.modalQuacInfo}>
                  <Text style={styles.modalQuacText}>
                    ID Destino Informado: {activeTransfer.destination_quac_id}
                  </Text>
                  <Text style={styles.modalQuacText}>Seu ID Quac: {identity.quac_id}</Text>
                </View>

                {transferError && (
                  <View style={styles.errorAlertBox}>
                    <Text style={styles.errorCodeTitle}>{transferError.code}</Text>
                    <Text style={styles.errorMessageText}>{transferError.message}</Text>
                    <Text style={styles.errorWarningText}>
                      ⚠️ O arquivo NÃO foi salvo neste dispositivo.
                    </Text>
                  </View>
                )}

                {!transferError && (
                  <View style={styles.progressContainer}>
                    <View style={styles.progressBarBg}>
                      <View style={[styles.progressBarFill, { width: `${transferProgress}%` }]} />
                    </View>
                    <Text style={styles.progressPercent}>{transferProgress}%</Text>

                    {transferSuccess && (
                      <Text style={styles.successMessage}>
                        ✓ Arquivo recebido e salvo com sucesso!
                      </Text>
                    )}
                  </View>
                )}
              </View>
            )}

            {savedFileUri && (
              <TouchableOpacity style={styles.shareButton} onPress={() => shareFile(savedFileUri)}>
                <Text style={styles.shareButtonText}>Salvar em Arquivos / Fotos / Compartilhar</Text>
              </TouchableOpacity>
            )}

            <TouchableOpacity
              style={[styles.modalCloseButton, isDownloading && { opacity: 0.5 }]}
              onPress={closeModal}
              disabled={!!isDownloading}
            >
              <Text style={styles.modalCloseButtonText}>
                {isDownloading ? 'Baixando...' : 'Fechar'}
              </Text>
            </TouchableOpacity>
          </View>
        </View>
      </Modal>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0f172a',
  },
  scrollContent: {
    padding: 20,
    paddingBottom: 40,
  },
  wizardContent: {
    flex: 1,
    justifyContent: 'center',
    padding: 24,
  },
  logo: {
    fontSize: 64,
    textAlign: 'center',
    marginBottom: 8,
  },
  title: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#f8fafc',
    textAlign: 'center',
  },
  subtitle: {
    fontSize: 16,
    color: '#fbbf24',
    textAlign: 'center',
    marginBottom: 32,
    fontWeight: '600',
  },
  card: {
    backgroundColor: '#1e293b',
    borderRadius: 16,
    padding: 24,
    borderWidth: 1,
    borderColor: '#334155',
  },
  label: {
    fontSize: 14,
    fontWeight: '600',
    color: '#cbd5e1',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#0f172a',
    borderRadius: 10,
    paddingHorizontal: 16,
    paddingVertical: 12,
    color: '#f8fafc',
    fontSize: 16,
    borderWidth: 1,
    borderColor: '#475569',
    marginBottom: 16,
  },
  messageInput: {
    backgroundColor: '#0f172a',
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 12,
    color: '#f8fafc',
    fontSize: 15,
    borderWidth: 1,
    borderColor: '#475569',
    minHeight: 80,
    textAlignVertical: 'top',
    marginBottom: 14,
  },
  buttonPrimary: {
    backgroundColor: '#f59e0b',
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  buttonPrimaryText: {
    color: '#0f172a',
    fontSize: 15,
    fontWeight: 'bold',
  },
  pickFileButton: {
    backgroundColor: '#0f172a',
    borderWidth: 1.5,
    borderColor: '#f59e0b',
    borderStyle: 'dashed',
    borderRadius: 12,
    paddingVertical: 18,
    alignItems: 'center',
    marginBottom: 12,
  },
  pickFileButtonText: {
    color: '#f59e0b',
    fontSize: 15,
    fontWeight: '700',
  },
  selectedFileCard: {
    backgroundColor: '#0f172a',
    borderRadius: 12,
    padding: 14,
    borderWidth: 1,
    borderColor: '#334155',
    marginBottom: 12,
  },
  selectedFileName: {
    color: '#f8fafc',
    fontSize: 14,
    fontWeight: 'bold',
    marginBottom: 4,
  },
  selectedFileSize: {
    color: '#94a3b8',
    fontSize: 12,
  },
  uploadingText: {
    color: '#f59e0b',
    fontSize: 12,
    textAlign: 'center',
    marginTop: 6,
    fontWeight: '600',
  },
  tabContainer: {
    flexDirection: 'row',
    backgroundColor: '#1e293b',
    borderRadius: 12,
    padding: 4,
    marginBottom: 16,
    borderWidth: 1,
    borderColor: '#334155',
  },
  tabButton: {
    flex: 1,
    paddingVertical: 10,
    alignItems: 'center',
    borderRadius: 8,
  },
  tabButtonActive: {
    backgroundColor: '#0f172a',
    borderWidth: 1,
    borderColor: '#f59e0b',
  },
  tabButtonText: {
    color: '#94a3b8',
    fontSize: 14,
    fontWeight: '600',
  },
  tabButtonTextActive: {
    color: '#f8fafc',
    fontWeight: 'bold',
  },
  buttonConnect: {
    backgroundColor: '#f59e0b',
    paddingHorizontal: 16,
    paddingVertical: 13,
    borderRadius: 10,
  },
  buttonConnectText: {
    color: '#0f172a',
    fontWeight: 'bold',
    fontSize: 14,
  },
  header: {
    marginBottom: 20,
  },
  headerTitleRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  duckEmoji: {
    fontSize: 40,
    marginRight: 12,
  },
  brandTitle: {
    fontSize: 22,
    fontWeight: 'bold',
    color: '#f8fafc',
    letterSpacing: 1,
  },
  brandTagline: {
    fontSize: 13,
    color: '#fbbf24',
    fontWeight: '600',
  },
  identityCard: {
    backgroundColor: '#1e293b',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
    borderWidth: 1,
    borderColor: '#334155',
  },
  identityHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  identityLabel: {
    fontSize: 11,
    fontWeight: '800',
    color: '#94a3b8',
    letterSpacing: 1,
  },
  badgeActive: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#064e3b',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 12,
  },
  badgeIdle: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#334155',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 12,
  },
  dotOnline: {
    width: 6,
    height: 6,
    borderRadius: 3,
    backgroundColor: '#10b981',
    marginRight: 6,
  },
  dotIdle: {
    width: 6,
    height: 6,
    borderRadius: 3,
    backgroundColor: '#94a3b8',
    marginRight: 6,
  },
  badgeText: {
    color: '#6ee7b7',
    fontSize: 11,
    fontWeight: '600',
  },
  badgeIdleText: {
    color: '#cbd5e1',
    fontSize: 11,
    fontWeight: '600',
  },
  deviceName: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#f8fafc',
    marginBottom: 12,
  },
  quacBox: {
    backgroundColor: '#0f172a',
    padding: 14,
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#334155',
  },
  quacLabel: {
    fontSize: 12,
    color: '#94a3b8',
    marginBottom: 2,
  },
  quacId: {
    fontSize: 28,
    fontWeight: '900',
    color: '#10b981',
    letterSpacing: 2,
  },
  sectionCard: {
    backgroundColor: '#1e293b',
    borderRadius: 16,
    padding: 18,
    marginBottom: 16,
    borderWidth: 1,
    borderColor: '#334155',
  },
  sectionTitleRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 10,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '700',
    color: '#f8fafc',
  },
  sectionSubtitle: {
    fontSize: 13,
    color: '#94a3b8',
    marginBottom: 12,
  },
  statusBanner: {
    flexDirection: 'row',
    alignItems: 'center',
    borderRadius: 12,
    padding: 14,
    marginBottom: 12,
    borderWidth: 1,
  },
  statusBannerGreen: {
    backgroundColor: '#064e3b',
    borderColor: '#059669',
  },
  statusBannerNeutral: {
    backgroundColor: '#0f172a',
    borderColor: '#334155',
  },
  dotLargeOnline: {
    width: 10,
    height: 10,
    borderRadius: 5,
    backgroundColor: '#10b981',
    marginRight: 12,
  },
  dotLargePulse: {
    width: 10,
    height: 10,
    borderRadius: 5,
    backgroundColor: '#f59e0b',
    marginRight: 12,
  },
  statusBannerTitle: {
    color: '#f8fafc',
    fontWeight: 'bold',
    fontSize: 14,
    marginBottom: 2,
  },
  statusBannerSub: {
    color: '#94a3b8',
    fontSize: 12,
  },
  autoActionRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 4,
  },
  buttonSecondary: {
    backgroundColor: '#334155',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
  },
  buttonSecondaryText: {
    color: '#f8fafc',
    fontSize: 12,
    fontWeight: '600',
  },
  buttonLink: {
    padding: 8,
  },
  buttonLinkText: {
    color: '#f59e0b',
    fontSize: 12,
    fontWeight: '600',
  },
  manualIpBox: {
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#334155',
  },
  manualIpLabel: {
    fontSize: 12,
    color: '#94a3b8',
    marginBottom: 6,
  },
  row: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  buttonRow: {
    flexDirection: 'row',
    gap: 10,
  },
  testButton: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 10,
    alignItems: 'center',
  },
  testButtonGreen: {
    backgroundColor: '#059669',
  },
  testButtonRed: {
    backgroundColor: '#dc2626',
  },
  testButtonText: {
    color: '#ffffff',
    fontWeight: '700',
    fontSize: 13,
  },
  emptyText: {
    color: '#64748b',
    fontSize: 13,
    fontStyle: 'italic',
    paddingVertical: 10,
  },
  transferItem: {
    backgroundColor: '#0f172a',
    padding: 12,
    borderRadius: 10,
    marginBottom: 8,
    borderWidth: 1,
  },
  itemAccepted: {
    borderColor: '#059669',
  },
  itemRejected: {
    borderColor: '#dc2626',
  },
  transferItemHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 4,
  },
  transferFileName: {
    color: '#f8fafc',
    fontSize: 14,
    fontWeight: 'bold',
    flex: 1,
    marginRight: 8,
  },
  transferStatusBadge: {
    fontSize: 11,
    fontWeight: '800',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 6,
  },
  badgeGreen: {
    backgroundColor: '#064e3b',
    color: '#34d399',
  },
  badgeRed: {
    backgroundColor: '#7f1d1d',
    color: '#f87171',
  },
  transferSender: {
    color: '#94a3b8',
    fontSize: 12,
  },
  transferErrorText: {
    color: '#fca5a5',
    fontSize: 11,
    marginTop: 4,
  },
  shareButton: {
    backgroundColor: '#f59e0b',
    paddingVertical: 12,
    borderRadius: 10,
    alignItems: 'center',
    marginBottom: 10,
  },
  shareButtonSmall: {
    alignSelf: 'flex-start',
    backgroundColor: '#f59e0b',
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 8,
    marginTop: 8,
  },
  shareButtonText: {
    color: '#0f172a',
    fontWeight: '700',
    fontSize: 13,
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.85)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modalCard: {
    backgroundColor: '#1e293b',
    borderRadius: 20,
    padding: 24,
    width: '100%',
    maxWidth: 400,
    borderWidth: 1,
    borderColor: '#475569',
  },
  modalTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#f8fafc',
    textAlign: 'center',
    marginBottom: 16,
  },
  modalBody: {
    marginBottom: 20,
  },
  modalFileName: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fbbf24',
    textAlign: 'center',
    marginBottom: 4,
  },
  modalSender: {
    fontSize: 13,
    color: '#cbd5e1',
    textAlign: 'center',
    marginBottom: 16,
  },
  modalQuacInfo: {
    backgroundColor: '#0f172a',
    padding: 10,
    borderRadius: 8,
    marginBottom: 16,
  },
  modalQuacText: {
    fontSize: 12,
    color: '#94a3b8',
  },
  errorAlertBox: {
    backgroundColor: '#450a0a',
    borderWidth: 1,
    borderColor: '#ef4444',
    padding: 14,
    borderRadius: 10,
  },
  errorCodeTitle: {
    color: '#f87171',
    fontWeight: '800',
    fontSize: 12,
    marginBottom: 4,
  },
  errorMessageText: {
    color: '#fecaca',
    fontSize: 13,
    lineHeight: 18,
    marginBottom: 8,
  },
  errorWarningText: {
    color: '#fca5a5',
    fontSize: 12,
    fontWeight: '700',
  },
  progressContainer: {
    alignItems: 'center',
  },
  progressBarBg: {
    width: '100%',
    height: 12,
    backgroundColor: '#0f172a',
    borderRadius: 6,
    overflow: 'hidden',
    marginBottom: 8,
  },
  progressBarFill: {
    height: '100%',
    backgroundColor: '#10b981',
  },
  progressPercent: {
    color: '#f8fafc',
    fontSize: 14,
    fontWeight: '700',
    marginBottom: 8,
  },
  successMessage: {
    color: '#34d399',
    fontWeight: '700',
    fontSize: 14,
    textAlign: 'center',
  },
  modalCloseButton: {
    backgroundColor: '#334155',
    paddingVertical: 12,
    borderRadius: 10,
    alignItems: 'center',
  },
  modalCloseButtonText: {
    color: '#f8fafc',
    fontWeight: '700',
    fontSize: 14,
  },
});
