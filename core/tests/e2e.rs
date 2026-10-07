//! Testes ponta-a-ponta: dois nós em 127.0.0.1 (portas efêmeras, sem multicast).

use std::time::Duration;

use ducker_core::model::{FileDto, PrepareUploadRequest, Protocol};
use ducker_core::{client::PeerClient, DuckerError, Identity, Node, NodeConfig, NodeEvent, Peer, SendOptions};
use tempfile::TempDir;

struct TestNode {
    node: Node,
    events: tokio::sync::broadcast::Receiver<NodeEvent>,
    save: TempDir,
}

async fn spawn(alias: &str, auto_accept: bool, protocol: Protocol) -> TestNode {
    let cfg_dir = tempfile::tempdir().unwrap();
    let save = tempfile::tempdir().unwrap();
    let identity = Identity::load_or_create(cfg_dir.path(), alias).unwrap();
    let mut config = NodeConfig::new(identity, save.path().to_path_buf());
    config.port = 0;
    config.enable_discovery = false;
    config.auto_accept = auto_accept;
    config.protocol = protocol;
    config.respond_timeout = Duration::from_secs(5);
    let (node, events) = Node::start(config).await.unwrap();
    TestNode { node, events, save }
}

async fn connect(from: &TestNode, to: &TestNode) -> Peer {
    from.node.connect("127.0.0.1".parse().unwrap(), to.node.port()).await.unwrap()
}

#[tokio::test]
async fn https_transfer_with_auto_accept_and_mutual_discovery() {
    let a = spawn("Remetente", false, Protocol::Https).await;
    let b = spawn("Receptor", true, Protocol::Https).await;

    let peer_b = connect(&a, &b).await;
    assert_eq!(peer_b.info.alias, "Receptor");
    assert_eq!(peer_b.protocol, Protocol::Https);
    assert_eq!(peer_b.info.quac_id, Some(b.node.identity().quac_id));
    // register é bidirecional: B agora conhece A
    assert!(b.node.peers().iter().any(|p| p.info.alias == "Remetente"));

    let src = tempfile::tempdir().unwrap();
    let file = src.path().join("ola.txt");
    std::fs::write(&file, b"Hello Ducker Network!").unwrap();

    let last = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let last2 = last.clone();
    a.node
        .send_files(&peer_b, vec![file], SendOptions::default(), move |p| {
            last2.store(p.total_sent, std::sync::atomic::Ordering::SeqCst)
        })
        .await
        .unwrap();

    assert_eq!(last.load(std::sync::atomic::Ordering::SeqCst), 21);
    let saved = b.save.path().join("ola.txt");
    assert_eq!(std::fs::read_to_string(saved).unwrap(), "Hello Ducker Network!");
}

#[tokio::test]
async fn folder_transfer_over_http() {
    let a = spawn("A", true, Protocol::Http).await;
    let b = spawn("B", true, Protocol::Http).await;
    let peer_b = connect(&a, &b).await;
    assert_eq!(peer_b.protocol, Protocol::Http);

    let src = tempfile::tempdir().unwrap();
    let dir = src.path().join("fotos");
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("1.jpg"), vec![1u8; 300_000]).unwrap();
    std::fs::write(dir.join("sub").join("2.jpg"), b"two").unwrap();

    a.node.send_files(&peer_b, vec![dir], SendOptions::default(), |_| {}).await.unwrap();

    assert_eq!(std::fs::read(b.save.path().join("fotos").join("1.jpg")).unwrap().len(), 300_000);
    assert_eq!(std::fs::read(b.save.path().join("fotos").join("sub").join("2.jpg")).unwrap(), b"two");
}

#[tokio::test]
async fn manual_accept_and_reject() {
    let a = spawn("A", true, Protocol::Https).await;
    let mut b = spawn("B", false, Protocol::Https).await;
    let peer_b = connect(&a, &b).await;

    let src = tempfile::tempdir().unwrap();
    let file = src.path().join("x.bin");
    std::fs::write(&file, b"data").unwrap();

    // Responder: recusa o primeiro pedido, aceita o segundo.
    let responder = b.node.clone();
    let handle = tokio::spawn(async move {
        let mut decisions = vec![true, false];
        while let Ok(ev) = b.events.recv().await {
            if let NodeEvent::IncomingRequest { session_id, auto_accepted, .. } = ev {
                assert!(!auto_accepted);
                let accept = decisions.pop().unwrap();
                assert!(responder.respond(&session_id, accept));
                if decisions.is_empty() {
                    break;
                }
            }
        }
        b.save
    });

    let r1 = a.node.send_files(&peer_b, vec![file.clone()], SendOptions::default(), |_| {}).await;
    assert!(matches!(r1, Err(DuckerError::Rejected)), "{r1:?}");
    a.node.send_files(&peer_b, vec![file], SendOptions::default(), |_| {}).await.unwrap();

    let save = handle.await.unwrap();
    assert!(save.path().join("x.bin").exists());
}

#[tokio::test]
async fn quac_mismatch_is_intercepted_and_nothing_saved() {
    let a = spawn("A", true, Protocol::Https).await;
    let b = spawn("B", true, Protocol::Https).await;
    let peer_b = connect(&a, &b).await;

    let src = tempfile::tempdir().unwrap();
    let file = src.path().join("secret.txt");
    std::fs::write(&file, b"nao salvar").unwrap();

    let wrong = if b.node.identity().quac_id == 99_999_999 { 10_000_000 } else { 99_999_999 };
    let opts = SendOptions { expected_quac: Some(wrong), ..Default::default() };
    let r = a.node.send_files(&peer_b, vec![file.clone()], opts, |_| {}).await;
    assert!(matches!(r, Err(DuckerError::DestinationIdMismatch)), "{r:?}");
    assert!(!b.save.path().join("secret.txt").exists());

    let ok = SendOptions { expected_quac: Some(b.node.identity().quac_id), ..Default::default() };
    a.node.send_files(&peer_b, vec![file], ok, |_| {}).await.unwrap();
    assert!(b.save.path().join("secret.txt").exists());
}

#[tokio::test]
async fn text_message_is_delivered_as_event() {
    let a = spawn("A", true, Protocol::Https).await;
    let mut b = spawn("B", false, Protocol::Https).await;
    let peer_b = connect(&a, &b).await;

    a.node.send_text(&peer_b, "olá pato 🦆", SendOptions::default()).await.unwrap();
    let text = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Ok(NodeEvent::TextReceived { text, sender }) = b.events.recv().await {
                assert_eq!(sender.alias, "A");
                return text;
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(text, "olá pato 🦆");
}

#[tokio::test]
async fn checksum_mismatch_returns_422_and_deletes_file() {
    let a = spawn("A", true, Protocol::Https).await;
    let b = spawn("B", true, Protocol::Https).await;
    let _ = connect(&a, &b).await;

    let client = PeerClient::new(Protocol::Https, "127.0.0.1".parse().unwrap(), b.node.port(), None, None).unwrap();
    let dto = FileDto {
        id: "f1".into(),
        file_name: "bad.txt".into(),
        size: 5,
        file_type: "application/octet-stream".into(),
        sha256: Some("00".repeat(32)),
        preview: None,
        metadata: None,
    };
    let req = PrepareUploadRequest { info: a.node.info(), files: [("f1".to_string(), dto)].into() };
    let resp = client.prepare_upload(&req, None, None).await.unwrap().unwrap();
    let token = resp.files.get("f1").unwrap();
    let r = client.upload_bytes(&resp.session_id, "f1", token, b"hello".to_vec()).await;
    assert!(matches!(r, Err(DuckerError::ChecksumMismatch)), "{r:?}");
    assert!(!b.save.path().join("bad.txt").exists());

    // Token inválido também é recusado
    let r2 = client.upload_bytes(&resp.session_id, "f1", "errado", b"x".to_vec()).await;
    assert!(r2.is_err());
}

#[tokio::test]
async fn info_endpoint_works() {
    let a = spawn("Pato", true, Protocol::Https).await;
    let client = PeerClient::new(Protocol::Https, "127.0.0.1".parse().unwrap(), a.node.port(), Some(a.node.identity().fingerprint), None).unwrap();
    let info = client.info().await.unwrap();
    assert_eq!(info.alias, "Pato");
    assert_eq!(info.fingerprint, a.node.identity().fingerprint);

    // Pinning com fingerprint errado deve falhar
    let bad = PeerClient::new(Protocol::Https, "127.0.0.1".parse().unwrap(), a.node.port(), Some("A".repeat(64)), None).unwrap();
    assert!(bad.info().await.is_err());
}
