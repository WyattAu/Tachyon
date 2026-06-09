// Offline Sync Queue — buffers CRDT updates when the editor is offline
//
// When the editor detects no server connectivity, it queues local CRDT
// updates (encoded as v1 bytes). When connectivity is restored, the
// queue is drained and updates are sent to the server in order.
//
// Design:
// - Each update is stored with a sequence number for ordering
// - Updates are merged opportunistically to reduce queue size
// - Queue is bounded to prevent unbounded memory growth
// - Status transitions: Offline -> Syncing -> Online

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, Transact, Update};

/// Maximum number of pending updates before forcing a merge.
const MAX_PENDING_BEFORE_MERGE: usize = 64;

/// Maximum queue capacity. Oldest updates are merged when exceeded.
const MAX_QUEUE_CAPACITY: usize = 512;

/// Connection status for the sync queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    /// No server connection. Updates are queued locally.
    Offline,
    /// Currently draining the queue to the server.
    Syncing,
    /// Connected and up-to-date.
    Online,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Offline => write!(f, "offline"),
            Self::Syncing => write!(f, "syncing"),
            Self::Online => write!(f, "online"),
        }
    }
}

/// A single queued CRDT update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedUpdate {
    /// Monotonically increasing sequence number.
    pub seq: u64,
    /// Encoded CRDT update (yrs v1 format).
    pub data: Vec<u8>,
    /// Timestamp when the update was created.
    pub timestamp_ms: u64,
}

/// Offline sync queue that buffers CRDT updates for later transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineSyncQueue {
    /// Pending updates waiting to be sent.
    updates: VecDeque<QueuedUpdate>,
    /// Next sequence number.
    next_seq: u64,
    /// Current sync status.
    status: SyncStatus,
    /// Total bytes queued.
    total_bytes: usize,
    /// Number of updates that have been successfully synced.
    synced_count: u64,
    /// Number of updates that failed to sync.
    failed_count: u64,
}

impl Default for OfflineSyncQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl OfflineSyncQueue {
    /// Create a new empty sync queue.
    pub fn new() -> Self {
        Self {
            updates: VecDeque::new(),
            next_seq: 1,
            status: SyncStatus::Offline,
            total_bytes: 0,
            synced_count: 0,
            failed_count: 0,
        }
    }

    /// Get the current sync status.
    pub fn status(&self) -> SyncStatus {
        self.status
    }

    /// Set the sync status.
    pub fn set_status(&mut self, status: SyncStatus) {
        self.status = status;
    }

    /// Number of pending updates in the queue.
    pub fn len(&self) -> usize {
        self.updates.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }

    /// Total bytes of queued updates.
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Number of successfully synced updates.
    pub fn synced_count(&self) -> u64 {
        self.synced_count
    }

    /// Number of updates that failed to sync.
    pub fn failed_count(&self) -> u64 {
        self.failed_count
    }

    /// Enqueue a local CRDT update for later transmission.
    ///
    /// If the queue exceeds `MAX_PENDING_BEFORE_MERGE`, adjacent updates
    /// are merged to reduce size. If the queue exceeds `MAX_QUEUE_CAPACITY`,
    /// the oldest updates are merged first.
    pub fn enqueue(&mut self, update: Vec<u8>, timestamp_ms: u64) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;

        let bytes = update.len();
        self.updates.push_back(QueuedUpdate {
            seq,
            data: update,
            timestamp_ms,
        });
        self.total_bytes += bytes;

        // Opportunistic merge when too many pending updates
        if self.updates.len() > MAX_PENDING_BEFORE_MERGE {
            self.merge_oldest_pair();
        }

        // Force merge from front if queue is too large
        while self.updates.len() > MAX_QUEUE_CAPACITY {
            self.merge_oldest_pair();
        }

        seq
    }

    /// Peek at the oldest update without removing it.
    pub fn peek(&self) -> Option<&QueuedUpdate> {
        self.updates.front()
    }

    /// Remove and return the oldest update (for sending to server).
    pub fn dequeue(&mut self) -> Option<QueuedUpdate> {
        if let Some(update) = self.updates.pop_front() {
            self.total_bytes = self.total_bytes.saturating_sub(update.data.len());
            Some(update)
        } else {
            None
        }
    }

    /// Drain up to `max` updates from the front of the queue.
    /// Returns the updates in order (oldest first).
    pub fn drain_batch(&mut self, max: usize) -> Vec<QueuedUpdate> {
        let count = max.min(self.updates.len());
        let batch: Vec<QueuedUpdate> = self.updates.drain(..count).collect();
        for update in &batch {
            self.total_bytes = self.total_bytes.saturating_sub(update.data.len());
        }
        batch
    }

    /// Mark the last dequeued batch as successfully synced.
    pub fn mark_synced(&mut self, count: u64) {
        self.synced_count += count;
    }

    /// Mark an update as failed. Increments failure counter and
    /// re-enqueues the update at the front.
    pub fn mark_failed(&mut self, update: QueuedUpdate) {
        self.failed_count += 1;
        self.total_bytes += update.data.len();
        self.updates.push_front(update);
    }

    /// Merge all pending updates into a single combined update.
    /// This is useful before sending a batch to the server.
    pub fn merge_all(&mut self) -> Option<Vec<u8>> {
        if self.updates.is_empty() {
            return None;
        }

        if self.updates.len() == 1 {
            let update = self.updates.pop_front()?;
            self.total_bytes = 0;
            return Some(update.data);
        }

        // Create a temporary doc to merge all updates
        let doc = Doc::new();
        let _text = doc.get_or_insert_text("content");

        // Apply all updates in order
        for queued in self.updates.drain(..) {
            if let Ok(update) = Update::decode_v1(&queued.data) {
                let mut txn = doc.transact_mut();
                let _ = txn.apply_update(update);
            }
        }

        // Encode the merged state
        let txn = doc.transact();
        let sv = yrs::StateVector::default();
        let merged = txn.encode_diff_v1(&sv);

        self.total_bytes = 0;

        if merged.is_empty() {
            None
        } else {
            Some(merged)
        }
    }

    /// Merge the two oldest updates into a single update.
    fn merge_oldest_pair(&mut self) {
        if self.updates.len() < 2 {
            return;
        }

        let Some(first) = self.updates.pop_front() else {
            return;
        };
        let Some(second) = self.updates.pop_front() else {
            self.updates.push_front(first);
            return;
        };

        // Decode both updates and apply to a temp doc
        let doc = Doc::new();
        let _text = doc.get_or_insert_text("content");

        // Apply first update
        if let Ok(update) = Update::decode_v1(&first.data) {
            let mut txn = doc.transact_mut();
            let _ = txn.apply_update(update);
        }

        // Apply second update
        if let Ok(update) = Update::decode_v1(&second.data) {
            let mut txn = doc.transact_mut();
            let _ = txn.apply_update(update);
        }

        // Encode merged state
        let txn = doc.transact();
        let sv = yrs::StateVector::default();
        let merged = txn.encode_diff_v1(&sv);

        // Update byte accounting
        self.total_bytes = self
            .total_bytes
            .saturating_sub(first.data.len())
            .saturating_sub(second.data.len());

        if !merged.is_empty() {
            self.total_bytes += merged.len();
            self.updates.push_front(QueuedUpdate {
                seq: first.seq, // Keep the earlier sequence number
                data: merged,
                timestamp_ms: first.timestamp_ms,
            });
        }
    }

    /// Get a summary of the queue state.
    pub fn summary(&self) -> SyncQueueSummary {
        SyncQueueSummary {
            pending_count: self.updates.len(),
            total_bytes: self.total_bytes,
            status: self.status,
            synced_count: self.synced_count,
            failed_count: self.failed_count,
            oldest_seq: self.updates.front().map(|u| u.seq),
            newest_seq: self.updates.back().map(|u| u.seq),
        }
    }
}

/// Summary of the sync queue state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncQueueSummary {
    pub pending_count: usize,
    pub total_bytes: usize,
    pub status: SyncStatus,
    pub synced_count: u64,
    pub failed_count: u64,
    pub oldest_seq: Option<u64>,
    pub newest_seq: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use yrs::{Doc, Text, Transact};

    fn timestamp() -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::now() as u64
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        }
    }

    fn make_update(content: &str) -> Vec<u8> {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        {
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 0, content);
        }
        let txn = doc.transact();
        let sv = yrs::StateVector::default();
        txn.encode_diff_v1(&sv)
    }

    #[test]
    fn test_enqueue_and_len() {
        let mut queue = OfflineSyncQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.status(), SyncStatus::Offline);

        queue.enqueue(make_update("hello"), timestamp());
        assert_eq!(queue.len(), 1);

        queue.enqueue(make_update("world"), timestamp());
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_dequeue_in_order() {
        let mut queue = OfflineSyncQueue::new();
        let seq1 = queue.enqueue(make_update("first"), timestamp());
        let seq2 = queue.enqueue(make_update("second"), timestamp());

        let Some(first) = queue.dequeue() else {
            panic!("expected first dequeue to return Some");
        };
        assert_eq!(first.seq, seq1);
        let Some(second) = queue.dequeue() else {
            panic!("expected second dequeue to return Some");
        };
        assert_eq!(second.seq, seq2);
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn test_drain_batch() {
        let mut queue = OfflineSyncQueue::new();
        for i in 0..10 {
            queue.enqueue(make_update(&format!("update-{i}")), timestamp());
        }

        let batch = queue.drain_batch(3);
        assert_eq!(batch.len(), 3);
        assert_eq!(queue.len(), 7);
    }

    #[test]
    fn test_drain_batch_more_than_available() {
        let mut queue = OfflineSyncQueue::new();
        queue.enqueue(make_update("only"), timestamp());

        let batch = queue.drain_batch(10);
        assert_eq!(batch.len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_mark_synced_and_failed() {
        let mut queue = OfflineSyncQueue::new();
        queue.enqueue(make_update("hello"), timestamp());
        queue.enqueue(make_update("world"), timestamp());

        let Some(_update) = queue.dequeue() else {
            panic!("expected first dequeue to return Some");
        };
        queue.mark_synced(1);
        assert_eq!(queue.synced_count(), 1);

        let Some(update2) = queue.dequeue() else {
            panic!("expected second dequeue to return Some");
        };
        queue.mark_failed(update2);
        assert_eq!(queue.failed_count(), 1);
        assert_eq!(queue.len(), 1); // Re-enqueued
    }

    #[test]
    fn test_status_transitions() {
        let mut queue = OfflineSyncQueue::new();
        assert_eq!(queue.status(), SyncStatus::Offline);

        queue.set_status(SyncStatus::Syncing);
        assert_eq!(queue.status(), SyncStatus::Syncing);

        queue.set_status(SyncStatus::Online);
        assert_eq!(queue.status(), SyncStatus::Online);
    }

    #[test]
    fn test_total_bytes_tracking() {
        let mut queue = OfflineSyncQueue::new();
        let update = make_update("hello world");
        let expected_bytes = update.len();

        queue.enqueue(update, timestamp());
        assert_eq!(queue.total_bytes(), expected_bytes);

        queue.dequeue();
        assert_eq!(queue.total_bytes(), 0);
    }

    #[test]
    fn test_summary() {
        let mut queue = OfflineSyncQueue::new();
        queue.enqueue(make_update("test"), timestamp());
        queue.mark_synced(5);

        let summary = queue.summary();
        assert_eq!(summary.pending_count, 1);
        assert_eq!(summary.synced_count, 5);
        assert_eq!(summary.failed_count, 0);
        assert!(summary.oldest_seq.is_some());
        assert!(summary.newest_seq.is_some());
    }

    #[test]
    fn test_merge_all_empty() {
        let mut queue = OfflineSyncQueue::new();
        assert!(queue.merge_all().is_none());
    }

    #[test]
    fn test_merge_all_single() {
        let mut queue = OfflineSyncQueue::new();
        let update = make_update("hello");
        queue.enqueue(update.clone(), timestamp());

        let merged = queue.merge_all();
        assert!(merged.is_some());
        assert!(queue.is_empty());
    }

    #[test]
    fn test_merge_all_multiple() {
        let mut queue = OfflineSyncQueue::new();
        queue.enqueue(make_update("hello "), timestamp());
        queue.enqueue(make_update("world"), timestamp());

        let merged = queue.merge_all();
        assert!(merged.is_some());
        assert!(queue.is_empty());
        assert_eq!(queue.total_bytes(), 0);
    }

    #[test]
    fn test_peek() {
        let mut queue = OfflineSyncQueue::new();
        assert!(queue.peek().is_none());

        let seq = queue.enqueue(make_update("hello"), timestamp());
        let Some(peeked) = queue.peek() else {
            panic!("expected peek to return Some after enqueue");
        };
        assert_eq!(peeked.seq, seq);
        assert_eq!(queue.len(), 1); // Not removed
    }

    #[test]
    fn test_sequence_numbers_monotonic() {
        let mut queue = OfflineSyncQueue::new();
        let seqs: Vec<u64> = (0..5)
            .map(|_| queue.enqueue(make_update("x"), timestamp()))
            .collect();

        // Verify monotonically increasing
        for i in 1..seqs.len() {
            assert!(seqs[i] > seqs[i - 1]);
        }
    }
}
