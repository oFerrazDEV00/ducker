import React, { useState, useEffect, useRef } from 'react';
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
} from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';

const STORAGE_KEY_IDENTITY = '@ducker_identity';
const DEFAULT_BRIDGE_PORT = 7876;

export default function App() {
  const [identity, setIdentity] = useState(null);
  const [deviceNameInput, setDeviceNameInput] = useState('');
  const [isConfiguring, setIsConfiguring] = useState(false);

  // Conexão com PC/Node
  const [nodeIp, setNodeIp] = useState('192.168.18.8');
  const [connectedNode, setConnectedNode] = useState(null);
  const [connectionStatus, setConnectionStatus] = useState('Desconectado');
  const wsRef = useRef(null);

  // Descoberta e transferências
  const [discoveredDevices, setDiscoveredDevices] = useState([]);
  const [receivedTransfers, setReceivedTransfers] = useState([]);

  // Modal de transferência ativa
  const [activeTransfer, setActiveTransfer] = useState(null);
  const [transferProgress, setTransferProgress] = useState(0);
  const [transferError, setTransferError] = useState(null);
  const [transferSuccess, setTransferSuccess] = useState(false);

  // Carregar identidade salva
  useEffect(() => {
    async function loadIdentity() {
      try {
        const stored = await AsyncStorage.getItem(STORAGE_KEY_IDENTITY);
        if (stored) {
          const parsed = JSON.parse(stored);
          setIdentity(parsed);
        } else {
          setIsConfiguring(true);
        }
      } catch (e) {
        setIsConfiguring(true);
      }
    }
    loadIdentity();
  }, []);

  // Conectar WebSocket ao nó quando houver identidade
  useEffect(() => {
    if (!identity || !nodeIp) return;

    let socket;
    try {
      const wsUrl = `ws://${nodeIp}:${DEFAULT_BRIDGE_PORT}/api/ws`;
      socket = new WebSocket(wsUrl);
      wsRef.current = socket;

      socket.onopen = () => {
        setConnectionStatus('Conectado');
        // Anunciar presença do celular
        const announceMsg = {
          type: 'mobile_announce',
          quac_id: identity.quac_id,
          device_name: identity.device_name,
        };
        socket.send(JSON.stringify(announceMsg));
      };

      socket.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          handleIncomingMessage(data);
        } catch (err) {
          console.error('Erro ao processar mensagem WS:', err);
        }
      };

      socket.onclose = () => {
        setConnectionStatus('Desconectado');
        setConnectedNode(null);
      };

      socket.onerror = () => {
        setConnectionStatus('Erro de conexão');
      };
    } catch (e) {
      setConnectionStatus('Erro ao criar socket');
    }

    return () => {
      if (socket) socket.close();
    };
  }, [identity, nodeIp]);

  // Tratar mensagens recebidas do PC/CLI
  function handleIncomingMessage(msg) {
    if (msg.type === 'transfer_status' && msg.status === 'CONNECTED') {
      setConnectedNode(msg.message);
    } else if (msg.type === 'transfer_request') {
      const req = msg;
      setActiveTransfer(req);
      setTransferProgress(0);
      setTransferError(null);
      setTransferSuccess(false);

      // Validação do ID Quac de destino
      const ownQuacId = identity.quac_id;
      const isMatch = req.destination_quac_id === ownQuacId;

      if (!isMatch) {
        // REJEITAR: DESTINATION_ID_MISMATCH
        const errorMsg =
          'Ocorreu um erro: a transferência foi interceptada porque o ID Quac de destino não corresponde a este dispositivo.';
        setTransferError({
          code: 'DESTINATION_ID_MISMATCH',
          message: errorMsg,
        });

        // Notificar PC que foi rejeitado
        if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
          const response = {
            type: 'handshake_response',
            status: 'REJECTED',
            reason: 'DESTINATION_ID_MISMATCH',
            message: errorMsg,
          };
          wsRef.current.send(JSON.stringify(response));
        }

        // Registrar no histórico como interceptado
        setReceivedTransfers((prev) => [
          {
            id: Date.now().toString(),
            fileName: req.file_name,
            sender: req.sender_name,
            senderQuac: req.sender_quac_id,
            status: 'INTERCEPTADO',
            error: errorMsg,
            date: new Date().toLocaleTimeString(),
          },
          ...prev,
        ]);
      } else {
        // ACEITAR
        if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
          const response = {
            type: 'handshake_response',
            status: 'ACCEPTED',
            reason: null,
            message: null,
          };
          wsRef.current.send(JSON.stringify(response));
        }

        // Simular progresso do download do stream
        let current = 0;
        const interval = setInterval(() => {
          current += 25;
          if (current >= 100) {
            clearInterval(interval);
            setTransferProgress(100);
            setTransferSuccess(true);

            // Registrar no histórico como recebido
            setReceivedTransfers((prev) => [
              {
                id: Date.now().toString(),
                fileName: req.file_name,
                sender: req.sender_name,
                senderQuac: req.sender_quac_id,
                status: 'RECEBIDO',
                size: (req.file_size / 1024).toFixed(1) + ' KB',
                date: new Date().toLocaleTimeString(),
              },
              ...prev,
            ]);
          } else {
            setTransferProgress(current);
          }
        }, 150);
      }
    }
  }

  // Salvar identidade inicial
  async function handleSaveIdentity() {
    const trimmed = deviceNameInput.trim();
    if (!trimmed) return;

    // Gerar ID Quac numérico de 8 dígitos (10000000 - 99999999)
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

  // Teste local de validação (Simulação imediata)
  function testTransferValidation(simulateMismatch = false) {
    if (!identity) return;

    const mockRequest = {
      type: 'transfer_request',
      protocol_version: 1,
      sender_name: 'Notebook do Fael',
      sender_quac_id: 84726193,
      destination_quac_id: simulateMismatch ? 99999999 : identity.quac_id,
      file_name: simulateMismatch ? 'documento_confidencial.pdf' : 'foto_fael.png',
      file_size: 204800,
    };

    handleIncomingMessage(mockRequest);
  }

  // TELA DE PRIMEIRA CONFIGURAÇÃO
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
              placeholder="Ex: Celular do Fael"
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

  // TELA PRINCIPAL DO DUCKER MOBILE
  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" />
      <ScrollView contentContainerStyle={styles.scrollContent}>
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

        {/* CARTÃO DE IDENTIDADE */}
        <View style={styles.identityCard}>
          <View style={styles.identityHeader}>
            <Text style={styles.identityLabel}>DISPOSITIVO LOCAL</Text>
            <View style={styles.badgeActive}>
              <View style={styles.dotOnline} />
              <Text style={styles.badgeText}>Ativo na Rede</Text>
            </View>
          </View>

          <Text style={styles.deviceName}>{identity.device_name}</Text>

          <View style={styles.quacBox}>
            <Text style={styles.quacLabel}>Seu ID Quac:</Text>
            <Text style={styles.quacId}>{identity.quac_id}</Text>
          </View>
        </View>

        {/* STATUS DO NÓ PC / REDE LOCAL */}
        <View style={styles.sectionCard}>
          <Text style={styles.sectionTitle}>Conexão com Notebook/PC</Text>
          <View style={styles.row}>
            <TextInput
              style={[styles.input, { flex: 1, marginRight: 8 }]}
              value={nodeIp}
              onChangeText={setNodeIp}
              placeholder="IP do Notebook (ex: 192.168.18.8)"
              placeholderTextColor="#64748b"
            />
            <View
              style={[
                styles.statusPill,
                connectionStatus === 'Conectado' ? styles.statusPillGreen : styles.statusPillYellow,
              ]}
            >
              <Text style={styles.statusPillText}>{connectionStatus}</Text>
            </View>
          </View>
          {connectedNode && <Text style={styles.connectedText}>✓ {connectedNode}</Text>}
        </View>

        {/* SIMULAÇÃO DE TESTES DO P0 */}
        <View style={styles.sectionCard}>
          <Text style={styles.sectionTitle}>Testes Rápidos do Protocolo P0</Text>
          <Text style={styles.sectionSubtitle}>
            Valide o comportamento exigido pela especificação com um toque:
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

        {/* HISTÓRICO DE TRANSFERÊNCIAS */}
        <View style={styles.sectionCard}>
          <Text style={styles.sectionTitle}>Transferências Recebidas</Text>

          {receivedTransfers.length === 0 ? (
            <Text style={styles.emptyText}>Nenhum arquivo recebido ainda.</Text>
          ) : (
            receivedTransfers.map((item) => (
              <View
                key={item.id}
                style={[
                  styles.transferItem,
                  item.status === 'INTERCEPTADO' ? styles.itemRejected : styles.itemAccepted,
                ]}
              >
                <View style={styles.transferItemHeader}>
                  <Text style={styles.transferFileName}>📦 {item.fileName}</Text>
                  <Text
                    style={[
                      styles.transferStatusBadge,
                      item.status === 'INTERCEPTADO' ? styles.badgeRed : styles.badgeGreen,
                    ]}
                  >
                    {item.status}
                  </Text>
                </View>
                <Text style={styles.transferSender}>
                  De: {item.sender} [Quac: {item.senderQuac}] • {item.date}
                </Text>
                {item.error && <Text style={styles.transferErrorText}>{item.error}</Text>}
              </View>
            ))
          )}
        </View>
      </ScrollView>

      {/* MODAL DE TRANSFERÊNCIA ATIVA / VALIDAÇÃO DE ID QUAC */}
      <Modal visible={!!activeTransfer} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalCard}>
            <Text style={styles.modalTitle}>
              {transferError ? 'Transferência Rejeitada' : 'Recebendo Arquivo'}
            </Text>

            {activeTransfer && (
              <View style={styles.modalBody}>
                <Text style={styles.modalFileName}>{activeTransfer.file_name}</Text>
                <Text style={styles.modalSender}>
                  De: {activeTransfer.sender_name} [Quac: {activeTransfer.sender_quac_id}]
                </Text>

                <View style={styles.modalQuacInfo}>
                  <Text style={styles.modalQuacText}>
                    ID Destino Informado: {activeTransfer.destination_quac_id}
                  </Text>
                  <Text style={styles.modalQuacText}>Seu ID Quac: {identity.quac_id}</Text>
                </View>

                {/* ERRO DE ID DIVERGENTE */}
                {transferError && (
                  <View style={styles.errorAlertBox}>
                    <Text style={styles.errorCodeTitle}>DESTINATION_ID_MISMATCH</Text>
                    <Text style={styles.errorMessageText}>{transferError.message}</Text>
                    <Text style={styles.errorWarningText}>
                      ⚠️ O arquivo NÃO foi salvo neste dispositivo.
                    </Text>
                  </View>
                )}

                {/* SUCESSO / PROGRESSO */}
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

            <TouchableOpacity style={styles.modalCloseButton} onPress={() => setActiveTransfer(null)}>
              <Text style={styles.modalCloseButtonText}>Fechar</Text>
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
  buttonPrimary: {
    backgroundColor: '#f59e0b',
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  buttonPrimaryText: {
    color: '#0f172a',
    fontSize: 16,
    fontWeight: 'bold',
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
  dotOnline: {
    width: 6,
    height: 6,
    borderRadius: 3,
    backgroundColor: '#10b981',
    marginRight: 6,
  },
  badgeText: {
    color: '#6ee7b7',
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
  sectionTitle: {
    fontSize: 16,
    fontWeight: '700',
    color: '#f8fafc',
    marginBottom: 6,
  },
  sectionSubtitle: {
    fontSize: 13,
    color: '#94a3b8',
    marginBottom: 12,
  },
  row: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusPill: {
    paddingHorizontal: 10,
    paddingVertical: 8,
    borderRadius: 8,
    marginBottom: 16,
  },
  statusPillGreen: {
    backgroundColor: '#065f46',
  },
  statusPillYellow: {
    backgroundColor: '#78350f',
  },
  statusPillText: {
    color: '#f8fafc',
    fontSize: 12,
    fontWeight: '700',
  },
  connectedText: {
    fontSize: 12,
    color: '#10b981',
    fontWeight: '600',
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
