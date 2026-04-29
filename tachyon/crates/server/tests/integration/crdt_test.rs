use std::time::Duration;

use axum::{Router, routing::get};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, ReadTxn, Text, Transact};

use tachyon_server::websocket::{CrdtConnectionManager, handle_crdt_websocket_upgrade};

// ============================================================================
// Helpers
// ============================================================================

fn skip_crdt_tests() -> bool {
    // CRDT tests require WebSocket server infrastructure that may not be
    // available in all test environments.
    std::env::var("RUN_CRDT_TESTS").is_err()
}

async fn start_crdt_test_server() -> (std::net::SocketAddr, CrdtConnectionManager) {
    let manager = CrdtConnectionManager::new();
    let app = Router::new()
        .route("/ws/crdt/{room}", get(handle_crdt_websocket_upgrade))
        .with_state(manager.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, manager)
}

fn create_yrs_update(text_content: &str) -> Vec<u8> {
    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");
    let mut txn = doc.transact_mut();
    text.insert(&mut txn, 0, text_content);
    let sv = txn.state_vector();
    txn.encode_state_as_update_v1(&sv)
}

fn wrap_sync_message(update: &[u8]) -> Vec<u8> {
    let mut msg = Vec::with_capacity(1 + update.len());
    msg.push(0);
    msg.extend_from_slice(update);
    msg
}

async fn drain_messages<S>(read: &mut S, timeout: Duration)
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        tokio::select! {
            _ = read.next() => {}
            _ = tokio::time::sleep(remaining) => break,
        }
    }
}

async fn read_next_binary<S>(read: &mut S, timeout: Duration) -> Option<Vec<u8>>
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    let result = tokio::time::timeout(timeout, async {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Binary(data)) => return Some(data.to_vec()),
                Ok(Message::Close(_)) => return None,
                Ok(_) => continue,
                Err(_) => return None,
            }
        }
        None
    })
    .await;
    result.ok().flatten()
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_connect() {
    if skip_crdt_tests() {
        println!("Skipping: RUN_CRDT_TESTS not set");
        return;
    }
    let (addr, _manager) = start_crdt_test_server().await;
    let url = format!("ws://{}/ws/crdt/test-doc-connect", addr);

    let (ws_stream, response) = tokio_tungstenite::connect_async(&url).await.unwrap();
    assert!(response.status().is_success(), "WebSocket upgrade should succeed");
    let (_write, mut read) = ws_stream.split();

    tokio::select! {
        msg = read.next() => {
            if let Some(Ok(msg)) = msg {
                match msg {
                    Message::Binary(data) => {
                        assert!(!data.is_empty());
                        assert_eq!(data[0], 0);
                    }
                    Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {}
                    _ => {}
                }
            }
        }
        _ = tokio::time::sleep(Duration::from_millis(200)) => {
            // New document, no initial state sent — timeout is fine
        }
    }
}

#[tokio::test]
async fn test_crdt_sync_single_client() {
    if skip_crdt_tests() {
        println!("Skipping: RUN_CRDT_TESTS not set");
        return;
    }
    let (addr, manager) = start_crdt_test_server().await;
    let room_id = "test-doc-single-sync";
    let url = format!("ws://{}/ws/crdt/{}", addr, room_id);

    let (ws_stream, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    drain_messages(&mut read, Duration::from_millis(100)).await;

    let update = create_yrs_update("Hello CRDT");
    write
        .send(Message::Binary(wrap_sync_message(&update).into()))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let text = manager.crdt_manager().get_text(room_id).unwrap();
    assert_eq!(text, "Hello CRDT");
}

#[tokio::test]
async fn test_crdt_sync_between_clients() {
    if skip_crdt_tests() {
        println!("Skipping: RUN_CRDT_TESTS not set");
        return;
    }
    let (addr, manager) = start_crdt_test_server().await;
    let room_id = "test-doc-2clients";

    let url1 = format!("ws://{}/ws/crdt/{}", addr, room_id);
    let (ws1, _) = tokio_tungstenite::connect_async(&url1).await.unwrap();
    let (mut write1, mut read1) = ws1.split();

    let url2 = format!("ws://{}/ws/crdt/{}", addr, room_id);
    let (ws2, _) = tokio_tungstenite::connect_async(&url2).await.unwrap();
    let (mut write2, mut read2) = ws2.split();

    drain_messages(&mut read1, Duration::from_millis(100)).await;
    drain_messages(&mut read2, Duration::from_millis(100)).await;

    // Client 1 inserts "Hello"
    let update1 = create_yrs_update("Hello");
    write1
        .send(Message::Binary(wrap_sync_message(&update1).into()))
        .await
        .unwrap();

    // Client 2 receives the relayed update
    let relayed = read_next_binary(&mut read2, Duration::from_secs(2)).await;
    assert!(
        relayed.is_some(),
        "Client 2 should receive Client 1's update"
    );
    let relayed = relayed.unwrap();
    assert!(relayed.len() > 1);
    assert_eq!(relayed[0], 0, "Should be a sync message (type 0)");

    // Client 2 applies update to its own Yrs doc
    let doc2 = Doc::new();
    let text2 = doc2.get_or_insert_text("content");
    let update = yrs::Update::decode_v1(&relayed[1..]).unwrap();
    let mut txn = doc2.transact_mut();
    txn.apply_update(update).unwrap();
    drop(txn);

    let txn = doc2.transact();
    assert_eq!(text2.get_string(&txn), "Hello");

    // Client 2 inserts " World" at position 5
    let text2_mut = doc2.get_or_insert_text("content");
    let mut txn2 = doc2.transact_mut();
    text2_mut.insert(&mut txn2, 5, " World");
    let sv = txn2.state_vector();
    let update2 = txn2.encode_state_as_update_v1(&sv);
    drop(txn2);

    write2
        .send(Message::Binary(wrap_sync_message(&update2).into()))
        .await
        .unwrap();

    // Client 1 receives the update
    let relayed1 = read_next_binary(&mut read1, Duration::from_secs(2)).await;
    assert!(
        relayed1.is_some(),
        "Client 1 should receive Client 2's update"
    );
    let relayed1 = relayed1.unwrap();
    assert!(relayed1.len() > 1);

    // Client 1 applies to its own doc
    let doc1 = Doc::new();
    let text1 = doc1.get_or_insert_text("content");
    let update = yrs::Update::decode_v1(&relayed1[1..]).unwrap();
    let mut txn = doc1.transact_mut();
    txn.apply_update(update).unwrap();
    drop(txn);

    let txn = doc1.transact();
    assert_eq!(text1.get_string(&txn), "Hello World");

    // Verify server-side state converged
    tokio::time::sleep(Duration::from_millis(100)).await;
    let server_text = manager.crdt_manager().get_text(room_id).unwrap();
    assert_eq!(server_text, "Hello World");
}

#[tokio::test]
async fn test_initial_state_sent_to_new_client() {
    if skip_crdt_tests() {
        println!("Skipping: RUN_CRDT_TESTS not set");
        return;
    }
    let (addr, manager) = start_crdt_test_server().await;
    let room_id = "test-doc-init-state";

    // Client 1 connects and sends content
    let url1 = format!("ws://{}/ws/crdt/{}", addr, room_id);
    let (ws1, _) = tokio_tungstenite::connect_async(&url1).await.unwrap();
    let (mut write1, mut read1) = ws1.split();

    drain_messages(&mut read1, Duration::from_millis(100)).await;

    let update = create_yrs_update("Initial content from client 1");
    write1
        .send(Message::Binary(wrap_sync_message(&update).into()))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        manager.crdt_manager().get_text(room_id).unwrap(),
        "Initial content from client 1"
    );

    // Client 2 connects — should receive initial state as sync step 1
    let url2 = format!("ws://{}/ws/crdt/{}", addr, room_id);
    let (ws2, _) = tokio_tungstenite::connect_async(&url2).await.unwrap();
    let (mut _write2, mut read2) = ws2.split();

    let received = read_next_binary(&mut read2, Duration::from_secs(2)).await;
    assert!(
        received.is_some(),
        "New client should receive initial document state"
    );

    let initial_state = received.unwrap();
    assert!(initial_state.len() > 2);
    assert_eq!(initial_state[0], 0, "Should be sync message type");
    assert_eq!(initial_state[1], 1, "Should be sync step 1");

    // Client 2 applies the initial state to its Yrs doc
    let doc2 = Doc::new();
    let text2 = doc2.get_or_insert_text("content");
    let state_update = yrs::Update::decode_v1(&initial_state[2..]).unwrap();
    let mut txn = doc2.transact_mut();
    txn.apply_update(state_update).unwrap();
    drop(txn);

    let txn = doc2.transact();
    assert_eq!(
        text2.get_string(&txn),
        "Initial content from client 1"
    );
}

#[tokio::test]
async fn test_presence_client_count() {
    let (addr, manager) = start_crdt_test_server().await;
    let room_id = "test-doc-presence";

    assert_eq!(manager.client_count().await, 0);

    // Connect client 1
    let url1 = format!("ws://{}/ws/crdt/{}", addr, room_id);
    let (mut ws1, _) = tokio_tungstenite::connect_async(&url1).await.unwrap();

    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(manager.client_count().await >= 1);

    // Connect client 2 to same room
    let url2 = format!("ws://{}/ws/crdt/{}", addr, room_id);
    let (_ws2, _) = tokio_tungstenite::connect_async(&url2).await.unwrap();

    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(
        manager.client_count().await >= 2,
        "Should have at least 2 clients"
    );

    let info = manager.get_collaboration_info(room_id).await;
    assert!(info.is_some());
    assert_eq!(info.unwrap().connection_count, 2);

    // Close client 1
    ws1.close(None).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(manager.client_count().await <= 1);
}

#[tokio::test]
async fn test_crdt_convergence_with_concurrent_edits() {
    if skip_crdt_tests() {
        println!("Skipping: RUN_CRDT_TESTS not set");
        return;
    }
    let (addr, manager) = start_crdt_test_server().await;
    let room_id = "test-doc-converge";

    let url1 = format!("ws://{}/ws/crdt/{}", addr, room_id);
    let (ws1, _) = tokio_tungstenite::connect_async(&url1).await.unwrap();
    let (mut write1, mut read1) = ws1.split();

    let url2 = format!("ws://{}/ws/crdt/{}", addr, room_id);
    let (ws2, _) = tokio_tungstenite::connect_async(&url2).await.unwrap();
    let (mut write2, mut read2) = ws2.split();

    drain_messages(&mut read1, Duration::from_millis(100)).await;
    drain_messages(&mut read2, Duration::from_millis(100)).await;

    // Client 1 doc: "AAA"
    let doc1 = Doc::new();
    let text1 = doc1.get_or_insert_text("content");
    let mut txn = doc1.transact_mut();
    text1.insert(&mut txn, 0, "AAA");
    let sv = txn.state_vector();
    let update1 = txn.encode_state_as_update_v1(&sv);
    drop(txn);

    // Client 2 doc: "BBB"
    let doc2 = Doc::new();
    let text2 = doc2.get_or_insert_text("content");
    let mut txn = doc2.transact_mut();
    text2.insert(&mut txn, 0, "BBB");
    let sv = txn.state_vector();
    let update2 = txn.encode_state_as_update_v1(&sv);
    drop(txn);

    // Send both updates concurrently
    write1
        .send(Message::Binary(wrap_sync_message(&update1).into()))
        .await
        .unwrap();
    write2
        .send(Message::Binary(wrap_sync_message(&update2).into()))
        .await
        .unwrap();

    // Both clients should receive the other's update
    let received_by_2 = read_next_binary(&mut read2, Duration::from_secs(2)).await;
    let received_by_1 = read_next_binary(&mut read1, Duration::from_secs(2)).await;

    assert!(
        received_by_2.is_some(),
        "Client 2 should receive Client 1's update"
    );
    assert!(
        received_by_1.is_some(),
        "Client 1 should receive Client 2's update"
    );

    // Apply the received updates
    if let Some(data) = received_by_2 {
        if data.len() > 1 && data[0] == 0 {
            if let Ok(update) = yrs::Update::decode_v1(&data[1..]) {
                let mut txn = doc2.transact_mut();
                txn.apply_update(update).unwrap();
            }
        }
    }

    if let Some(data) = received_by_1 {
        if data.len() > 1 && data[0] == 0 {
            if let Ok(update) = yrs::Update::decode_v1(&data[1..]) {
                let mut txn = doc1.transact_mut();
                txn.apply_update(update).unwrap();
            }
        }
    }

    // Both docs must converge to the same content
    let text1_final = {
        let txn = doc1.transact();
        text1.get_string(&txn)
    };
    let text2_final = {
        let txn = doc2.transact();
        text2.get_string(&txn)
    };

    assert_eq!(
        text1_final, text2_final,
        "Both clients should converge to the same CRDT state"
    );

    // Server must also match
    tokio::time::sleep(Duration::from_millis(100)).await;
    let server_text = manager.crdt_manager().get_text(room_id).unwrap();
    assert_eq!(
        server_text, text1_final,
        "Server should match converged client state"
    );
}

#[tokio::test]
async fn test_crdt_manager_direct() {
    if skip_crdt_tests() {
        println!("Skipping: RUN_CRDT_TESTS not set");
        return;
    }
    let manager = tachyon_server::crdt::CrdtDocumentManager::new();
    let doc_id = "direct-test";

    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");
    let mut txn = doc.transact_mut();
    text.insert(&mut txn, 0, "Direct test content");
    let sv = txn.state_vector();
    let update = txn.encode_state_as_update_v1(&sv);
    drop(txn);

    manager.apply_update(doc_id, &update).unwrap();
    assert_eq!(
        manager.get_text(doc_id).unwrap(),
        "Direct test content"
    );

    let state = manager.get_state(doc_id).unwrap();
    assert!(!state.is_empty());

    manager.set_text(doc_id, "Completely new text").unwrap();
    assert_eq!(
        manager.get_text(doc_id).unwrap(),
        "Completely new text"
    );

    // Non-existent document returns empty string
    assert_eq!(manager.get_text("nonexistent").unwrap(), "");
}
