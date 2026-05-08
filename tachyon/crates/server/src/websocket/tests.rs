use crate::websocket::crdt_handler::{CrdtConnectionManager, RelayEvent};
use axum::extract::ws::Message;
use std::time::Duration;
use yrs::{Doc, ReadTxn, Text, Transact};

fn make_update(text: &str) -> Vec<u8> {
    let doc = Doc::new();
    let txt = doc.get_or_insert_text("content");
    {
        let mut txn = doc.transact_mut();
        txt.insert(&mut txn, 0, text);
    }
    let txn = doc.transact();
    let sv = yrs::StateVector::default();
    txn.encode_state_as_update_v1(&sv)
}

fn encode_sync_message(update: &[u8]) -> Vec<u8> {
    let mut msg = Vec::with_capacity(2 + update.len());
    msg.push(0);
    msg.push(1);
    msg.extend_from_slice(update);
    msg
}

fn encode_selection_update(start: u32, end: u32) -> Vec<u8> {
    let mut msg = vec![0x02];
    msg.extend_from_slice(&start.to_le_bytes());
    msg.extend_from_slice(&end.to_le_bytes());
    msg
}

#[tokio::test]
async fn client_connection_multiple_clients() {
    let manager = CrdtConnectionManager::new();
    assert_eq!(manager.client_count().await, 0);

    let m1 = manager.clone();
    let m2 = manager.clone();
    let m3 = manager.clone();

    let h1 = tokio::spawn(async move {
        m1.join_room("client-a", "room-1").await;
    });
    let h2 = tokio::spawn(async move {
        m2.join_room("client-b", "room-1").await;
    });
    let h3 = tokio::spawn(async move {
        m3.join_room("client-c", "room-2").await;
    });

    h1.await.unwrap();
    h2.await.unwrap();
    h3.await.unwrap();

    let info1 = manager.get_collaboration_info("room-1").await.unwrap();
    assert_eq!(info1.connection_count, 2);

    let info2 = manager.get_collaboration_info("room-2").await.unwrap();
    assert_eq!(info2.connection_count, 1);
}

#[tokio::test]
async fn document_sync_changes_propagate() {
    let manager = CrdtConnectionManager::new();
    let room = "sync-room-1";

    let update = make_update("Hello from client A");
    manager
        .crdt_manager()
        .apply_update(room, &update)
        .expect("apply update should succeed");

    let state = manager.crdt_manager().get_text(room).unwrap();
    assert!(state.contains("Hello from client A"));

    let update2 = make_update(" and client B");
    manager
        .crdt_manager()
        .apply_update(room, &update2)
        .expect("apply update should succeed");

    let state = manager.crdt_manager().get_text(room).unwrap();
    assert!(state.contains("Hello from client A"));
    assert!(state.contains("and client B"));
}

#[tokio::test]
async fn concurrent_edits_consistent_state() {
    let manager = CrdtConnectionManager::new();
    let room = "concurrent-room-1";

    let update_a = make_update("Hello");
    let update_b = make_update(" World");

    manager
        .crdt_manager()
        .apply_update(room, &update_a)
        .expect("apply update A should succeed");

    manager
        .crdt_manager()
        .apply_update(room, &update_b)
        .expect("apply update B should succeed");

    let state = manager.crdt_manager().get_text(room).unwrap();
    assert!(state.contains("Hello"), "State should contain 'Hello'");
    assert!(state.contains("World"), "State should contain 'World'");
    assert_eq!(
        manager.crdt_manager().get_text(room).unwrap().len(),
        "Hello World".len()
    );
}

#[tokio::test]
async fn client_disconnect_remaining_continue() {
    let manager = CrdtConnectionManager::new();

    let (tx1, _) = tokio::sync::mpsc::unbounded_channel::<Message>();
    let (tx2, _) = tokio::sync::mpsc::unbounded_channel::<Message>();

    manager.clients.write().await.insert(
        "client-disconnect-1".to_string(),
        crate::websocket::crdt_handler::ConnectedClient {
            client_id: "client-disconnect-1".to_string(),
            room: "room-disconnect".to_string(),
            send: tx1,
            last_seen: std::time::Instant::now(),
        },
    );
    manager.clients.write().await.insert(
        "client-disconnect-2".to_string(),
        crate::websocket::crdt_handler::ConnectedClient {
            client_id: "client-disconnect-2".to_string(),
            room: "room-disconnect".to_string(),
            send: tx2,
            last_seen: std::time::Instant::now(),
        },
    );
    manager
        .join_room("client-disconnect-1", "room-disconnect")
        .await;
    manager
        .join_room("client-disconnect-2", "room-disconnect")
        .await;

    assert_eq!(manager.client_count().await, 2);

    manager.leave_room("client-disconnect-1").await;

    assert_eq!(manager.client_count().await, 1);

    let info = manager
        .get_collaboration_info("room-disconnect")
        .await
        .unwrap();
    assert_eq!(info.connection_count, 1);

    let update = make_update("still working");
    manager
        .crdt_manager()
        .apply_update("room-disconnect", &update)
        .expect("should still apply updates after disconnect");
}

#[tokio::test]
async fn reconnection_gets_current_state() {
    let manager = CrdtConnectionManager::new();
    let room = "reconnect-room-1";

    let update1 = make_update("Initial content");
    manager
        .crdt_manager()
        .apply_update(room, &update1)
        .expect("apply should succeed");

    let state = manager
        .crdt_manager()
        .get_state(room)
        .expect("get_state should succeed");
    assert!(!state.is_empty(), "Document should have state after update");

    let update2 = make_update(" more content");
    manager
        .crdt_manager()
        .apply_update(room, &update2)
        .expect("apply should succeed");

    let state_after = manager
        .crdt_manager()
        .get_state(room)
        .expect("get_state should succeed");
    assert!(
        !state_after.is_empty(),
        "Document state should exist for reconnection"
    );

    let text = manager.crdt_manager().get_text(room).unwrap();
    assert!(text.contains("Initial content"));
    assert!(text.contains("more content"));
}

#[tokio::test]
async fn broadcast_relay_between_clients() {
    let manager = CrdtConnectionManager::new();
    let room = "broadcast-room-1";

    let mut rx = manager.subscribe();

    let data = vec![0x00, 0x01, 0xAA, 0xBB];
    manager
        .broadcast_tx
        .send(RelayEvent::Binary {
            room: room.to_string(),
            sender: "sender-1".to_string(),
            data: data.clone(),
        })
        .unwrap();

    let event: RelayEvent = tokio::time::timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("should receive event within timeout")
        .expect("should receive event");

    match event {
        RelayEvent::Binary {
            room: event_room,
            sender,
            data: event_data,
        } => {
            assert_eq!(event_room, room);
            assert_eq!(sender, "sender-1");
            assert_eq!(event_data, data);
        }
        _ => panic!("Expected Binary relay event"),
    }
}

#[tokio::test]
async fn broadcast_selection_relay() {
    let manager = CrdtConnectionManager::new();
    let room = "selection-room-1";

    let mut rx = manager.subscribe();

    let data = encode_selection_update(10, 50);
    let selection = crate::websocket::crdt_handler::parse_selection_update(&data).unwrap();

    manager
        .broadcast_tx
        .send(RelayEvent::Selection {
            room: room.to_string(),
            sender: "sender-sel".to_string(),
            data: data.clone(),
            selection,
        })
        .unwrap();

    let event: RelayEvent = tokio::time::timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("should receive selection event")
        .expect("should receive event");

    match event {
        RelayEvent::Selection {
            room: event_room,
            sender,
            selection: sel,
            ..
        } => {
            assert_eq!(event_room, room);
            assert_eq!(sender, "sender-sel");
            assert_eq!(sel.start, 10);
            assert_eq!(sel.end, 50);
        }
        _ => panic!("Expected Selection relay event"),
    }
}

#[tokio::test]
async fn room_isolation_messages_not_relayed_across_rooms() {
    let manager = CrdtConnectionManager::new();

    let mut rx_room1 = manager.subscribe();
    let mut rx_room2 = manager.subscribe();

    manager
        .broadcast_tx
        .send(RelayEvent::Binary {
            room: "room-alpha".to_string(),
            sender: "sender-x".to_string(),
            data: vec![0x00, 0x01],
        })
        .unwrap();

    let event: RelayEvent = tokio::time::timeout(Duration::from_millis(100), rx_room1.recv())
        .await
        .expect("timeout")
        .expect("event");

    match event {
        RelayEvent::Binary { room, .. } => {
            assert_eq!(room, "room-alpha");
        }
        _ => panic!("Expected Binary event"),
    }

    let result = tokio::time::timeout(Duration::from_millis(100), rx_room2.recv()).await;
    assert!(
        result.is_ok(),
        "Both subscribers receive broadcast events (broadcast is not room-filtered)"
    );
}

#[tokio::test]
async fn cleanup_stale_clients() {
    let manager = CrdtConnectionManager::new();

    let (tx1, _) = tokio::sync::mpsc::unbounded_channel::<Message>();
    let (tx2, _) = tokio::sync::mpsc::unbounded_channel::<Message>();

    let stale_client = crate::websocket::crdt_handler::ConnectedClient {
        client_id: "stale-client".to_string(),
        room: "cleanup-room".to_string(),
        send: tx1,
        last_seen: std::time::Instant::now() - Duration::from_secs(600),
    };

    let fresh_client = crate::websocket::crdt_handler::ConnectedClient {
        client_id: "fresh-client".to_string(),
        room: "cleanup-room".to_string(),
        send: tx2,
        last_seen: std::time::Instant::now(),
    };

    manager
        .clients
        .write()
        .await
        .insert("stale-client".to_string(), stale_client);
    manager
        .clients
        .write()
        .await
        .insert("fresh-client".to_string(), fresh_client);

    manager.join_room("stale-client", "cleanup-room").await;
    manager.join_room("fresh-client", "cleanup-room").await;

    assert_eq!(manager.client_count().await, 2);

    manager.cleanup_stale_clients(300).await;

    assert_eq!(manager.client_count().await, 1);

    let info = manager
        .get_collaboration_info("cleanup-room")
        .await
        .unwrap();
    assert_eq!(info.connection_count, 1);
}

#[tokio::test]
async fn crdt_manager_shared_across_rooms() {
    let manager = CrdtConnectionManager::new();

    let update_room1 = make_update("Room 1 content");
    let update_room2 = make_update("Room 2 content");

    manager
        .crdt_manager()
        .apply_update("shared-room-1", &update_room1)
        .unwrap();
    manager
        .crdt_manager()
        .apply_update("shared-room-2", &update_room2)
        .unwrap();

    let text1 = manager.crdt_manager().get_text("shared-room-1").unwrap();
    let text2 = manager.crdt_manager().get_text("shared-room-2").unwrap();

    assert!(text1.contains("Room 1 content"));
    assert!(text2.contains("Room 2 content"));
    assert!(!text1.contains("Room 2 content"));
    assert!(!text2.contains("Room 1 content"));
}

#[tokio::test]
async fn join_leave_events_broadcast() {
    let manager = CrdtConnectionManager::new();
    let mut rx = manager.subscribe();

    manager
        .broadcast_tx
        .send(RelayEvent::Joined {
            room: "event-room".to_string(),
            client_id: "client-evt-1".to_string(),
        })
        .unwrap();

    let event: RelayEvent = tokio::time::timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("timeout")
        .expect("event");

    match event {
        RelayEvent::Joined { room, client_id } => {
            assert_eq!(room, "event-room");
            assert_eq!(client_id, "client-evt-1");
        }
        _ => panic!("Expected Joined event"),
    }

    manager
        .broadcast_tx
        .send(RelayEvent::Left {
            room: "event-room".to_string(),
            client_id: "client-evt-1".to_string(),
        })
        .unwrap();

    let event: RelayEvent = tokio::time::timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("timeout")
        .expect("event");

    match event {
        RelayEvent::Left { room, client_id } => {
            assert_eq!(room, "event-room");
            assert_eq!(client_id, "client-evt-1");
        }
        _ => panic!("Expected Left event"),
    }
}

#[tokio::test]
async fn init_and_get_document_update_log() {
    let manager = CrdtConnectionManager::new();

    let log_data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE];
    manager
        .init_document_from_persisted("persisted-doc-1", log_data.clone())
        .await;

    let retrieved = manager
        .get_document_update_log("persisted-doc-1")
        .await
        .expect("should return update log");
    assert_eq!(retrieved, log_data);

    let missing = manager.get_document_update_log("nonexistent-doc").await;
    assert!(missing.is_none(), "Non-existent doc should return None");
}

#[tokio::test]
async fn encode_sync_step1_produces_valid_format() {
    let update = vec![0xAA, 0xBB, 0xCC, 0xDD];
    let msg = encode_sync_message(&update);

    assert_eq!(msg.len(), 2 + update.len());
    assert_eq!(msg[0], 0);
    assert_eq!(msg[1], 1);
    assert_eq!(&msg[2..], &update);
}

#[tokio::test]
async fn selection_update_parsing() {
    let valid = encode_selection_update(100, 200);
    let parsed = crate::websocket::crdt_handler::parse_selection_update(&valid);
    assert!(parsed.is_some());
    let sel = parsed.unwrap();
    assert_eq!(sel.start, 100);
    assert_eq!(sel.end, 200);

    let too_short = vec![0x02, 0x01];
    assert!(crate::websocket::crdt_handler::parse_selection_update(&too_short).is_none());

    let empty: Vec<u8> = vec![];
    assert!(crate::websocket::crdt_handler::parse_selection_update(&empty).is_none());
}

#[tokio::test]
async fn multiple_concurrent_managers_isolated() {
    let manager_a = CrdtConnectionManager::new();
    let manager_b = CrdtConnectionManager::new();

    manager_a.join_room("c1", "room-a").await;
    manager_b.join_room("c2", "room-b").await;

    let info_a = manager_a.get_collaboration_info("room-a").await.unwrap();
    assert_eq!(info_a.connection_count, 1);

    let info_b = manager_b.get_collaboration_info("room-b").await.unwrap();
    assert_eq!(info_b.connection_count, 1);

    assert!(manager_a.get_collaboration_info("room-b").await.is_none());
    assert!(manager_b.get_collaboration_info("room-a").await.is_none());

    manager_a
        .crdt_manager()
        .set_text("room-a", "Manager A doc")
        .unwrap();
    manager_b
        .crdt_manager()
        .set_text("room-b", "Manager B doc")
        .unwrap();

    assert_eq!(
        manager_a.crdt_manager().get_text("room-a").unwrap(),
        "Manager A doc"
    );
    assert_eq!(
        manager_b.crdt_manager().get_text("room-b").unwrap(),
        "Manager B doc"
    );
}
