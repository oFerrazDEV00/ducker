use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use ducker_core::{
    DeviceIdentity, DuckerError, FileReceiver, FileSender,
};

#[tokio::test]
async fn test_cli_to_mobile_success_flow() {
    // 1. Mobile inicia com nome e ID Quac
    let mobile_identity = DeviceIdentity::with_id("Celular do Fael", 12345678);
    // 2. Notebook/PC inicia com nome e ID Quac
    let pc_identity = DeviceIdentity::with_id("Notebook do Fael", 84726193);

    let temp_dir = tempfile::tempdir().unwrap();
    let mobile_save_dir = temp_dir.path().join("mobile_storage");

    // Porta efêmera para teste
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    // Mobile inicia servidor receptor
    let (receiver, mut events) = FileReceiver::with_port(mobile_identity.clone(), mobile_save_dir.clone(), port);
    receiver.start().await.unwrap();

    // Arquivo no PC a ser enviado
    let test_file = temp_dir.path().join("foto.png");
    tokio::fs::write(&test_file, b"IMAGEM_BYTES_DUCKER_12345").await.unwrap();

    let target_addr = format!("127.0.0.1:{}", port);
    let progress_triggered = Arc::new(AtomicBool::new(false));
    let progress_clone = Arc::clone(&progress_triggered);

    // PC envia para o Celular com o ID Quac correto (12345678)
    let send_result = FileSender::send_file(
        &target_addr,
        12345678,
        &pc_identity,
        &test_file,
        move |sent, total| {
            if sent == total {
                progress_clone.store(true, Ordering::SeqCst);
            }
        },
    ).await;

    assert!(send_result.is_ok(), "Transferência para o Celular deve ser bem-sucedida");
    assert!(progress_triggered.load(Ordering::SeqCst), "Progresso deve atingir 100%");

    // Validar que o Mobile recebeu e salvou o arquivo
    let received_file = mobile_save_dir.join("foto.png");
    assert!(received_file.exists(), "O arquivo foto.png deve estar salvo no dispositivo de destino");
    let content = tokio::fs::read(&received_file).await.unwrap();
    assert_eq!(content, b"IMAGEM_BYTES_DUCKER_12345");

    // Validar evento de conclusão
    let event = events.recv().await.unwrap();
    println!("Evento recebido no Mobile: {:?}", event);
}

#[tokio::test]
async fn test_cli_to_mobile_id_mismatch_interception() {
    // Receptor tem ID 12345678
    let mobile_identity = DeviceIdentity::with_id("Celular do Fael", 12345678);
    let pc_identity = DeviceIdentity::with_id("Notebook do Fael", 84726193);

    let temp_dir = tempfile::tempdir().unwrap();
    let mobile_save_dir = temp_dir.path().join("mobile_storage_mismatch");

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let (receiver, _events) = FileReceiver::with_port(mobile_identity.clone(), mobile_save_dir.clone(), port);
    receiver.start().await.unwrap();

    let secret_file = temp_dir.path().join("arquivo_sigiloso.pdf");
    tokio::fs::write(&secret_file, b"DADOS_CONFIDENCIAIS").await.unwrap();

    let target_addr = format!("127.0.0.1:{}", port);

    // PC tenta enviar especificando o ID ERRADO (99999999 != 12345678)
    let send_result = FileSender::send_file(
        &target_addr,
        99999999, // ID incorreto
        &pc_identity,
        &secret_file,
        |_sent, _total| {},
    ).await;

    // Deve falhar com DESTINATION_ID_MISMATCH
    assert!(send_result.is_err(), "Transferência com ID divergente DEVE falhar");
    match send_result.unwrap_err() {
        DuckerError::DestinationIdMismatch => {
            println!("✓ DESTINATION_ID_MISMATCH validado com sucesso.");
        }
        other => panic!("Esperado erro DestinationIdMismatch, mas retornou: {:?}", other),
    }

    // REGRA DE OURO DO P0: NENHUM arquivo deve ser salvo quando o ID for incorreto
    let saved_file = mobile_save_dir.join("arquivo_sigiloso.pdf");
    assert!(
        !saved_file.exists(),
        "REGRA CRÍTICA P0: Nenhum arquivo deve ser salvo quando o ID Quac não corresponder!"
    );
}
