use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Message types for chat broadcast channel.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ChatMessage {
    /// Regular chat message.
    Public { sender: String, text: String },
    /// /me emote action.
    Action { sender: String, action: String },
    /// System announcement (errors, capacity, etc.).
    System { text: String },
    /// Private message between two users.
    Direct { from: String, to: String, text: String },
    /// User joined chat.
    Join { handle: String },
    /// User left chat.
    Leave { handle: String },
    /// Page notification (triggers bell on recipient).
    Page { from: String, to: String },
}

/// Manages the chat room with broadcast channel for real-time message distribution.
///
/// Uses tokio::sync::broadcast for 1-to-many message delivery without manual iteration.
/// Tracks participants with RwLock<HashMap> for thread-safe access.
#[derive(Clone)]
pub struct ChatManager {
    tx: broadcast::Sender<ChatMessage>,
    participants: Arc<RwLock<HashMap<i64, String>>>, // user_id -> handle
    capacity: usize,
}

impl ChatManager {
    /// Create a new ChatManager with the given participant capacity.
    ///
    /// The broadcast channel itself uses a buffer of 100 messages.
    pub fn new(capacity: usize) -> Self {
        // Broadcast channel buffer size (messages in flight, not participants)
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            participants: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }

    /// Subscribe to the chat broadcast channel.
    ///
    /// Returns a Receiver that will receive all future messages.
    pub fn subscribe(&self) -> broadcast::Receiver<ChatMessage> {
        self.tx.subscribe()
    }

    /// Broadcast a message to all subscribers.
    ///
    /// Ignores errors when no subscribers are connected.
    pub fn broadcast(&self, msg: ChatMessage) {
        let _ = self.tx.send(msg);
    }

    /// Join the chat room.
    ///
    /// Returns Ok(()) if joined successfully, Err(message) if at capacity.
    pub async fn join(&self, user_id: i64, handle: String) -> Result<(), String> {
        let mut participants = self.participants.write().await;

        // Check if already in chat
        if participants.contains_key(&user_id) {
            return Ok(()); // Already in chat, no-op
        }

        // Check capacity
        if participants.len() >= self.capacity {
            return Err("Chat room is at capacity. Please try again later.".to_string());
        }

        participants.insert(user_id, handle);
        Ok(())
    }

    /// Leave the chat room.
    pub async fn leave(&self, user_id: i64) {
        let mut participants = self.participants.write().await;
        participants.remove(&user_id);
    }

    /// Get list of all participant handles.
    pub async fn get_participants(&self) -> Vec<String> {
        let participants = self.participants.read().await;
        participants.values().cloned().collect()
    }

    /// Check if a user is currently in chat.
    #[allow(dead_code)]
    pub async fn is_in_chat(&self, user_id: i64) -> bool {
        let participants = self.participants.read().await;
        participants.contains_key(&user_id)
    }

    /// Get current participant count.
    #[allow(dead_code)]
    pub async fn get_participant_count(&self) -> usize {
        let participants = self.participants.read().await;
        participants.len()
    }

    /// Get user_id for a handle (reverse lookup for /msg command).
    #[allow(dead_code)]
    pub async fn get_handle_user_id(&self, handle: &str) -> Option<i64> {
        let participants = self.participants.read().await;
        let handle_lower = handle.to_lowercase();
        participants
            .iter()
            .find(|(_, h)| h.to_lowercase() == handle_lower)
            .map(|(&id, _)| id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_creates_empty_manager() {
        let mgr = ChatManager::new(10);
        assert_eq!(mgr.get_participant_count().await, 0);
    }

    #[tokio::test]
    async fn join_adds_participant() {
        let mgr = ChatManager::new(10);

        mgr.join(1, "Alice".to_string()).await.unwrap();

        assert!(mgr.is_in_chat(1).await);
        assert_eq!(mgr.get_participant_count().await, 1);
    }

    #[tokio::test]
    async fn join_respects_capacity() {
        let mgr = ChatManager::new(2);

        mgr.join(1, "Alice".to_string()).await.unwrap();
        mgr.join(2, "Bob".to_string()).await.unwrap();
        let result = mgr.join(3, "Carol".to_string()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("capacity"));
    }

    #[tokio::test]
    async fn join_is_idempotent() {
        let mgr = ChatManager::new(10);

        mgr.join(1, "Alice".to_string()).await.unwrap();
        mgr.join(1, "Alice".to_string()).await.unwrap(); // Should not error

        assert_eq!(mgr.get_participant_count().await, 1);
    }

    #[tokio::test]
    async fn leave_removes_participant() {
        let mgr = ChatManager::new(10);

        mgr.join(1, "Alice".to_string()).await.unwrap();
        assert!(mgr.is_in_chat(1).await);

        mgr.leave(1).await;
        assert!(!mgr.is_in_chat(1).await);
    }

    #[tokio::test]
    async fn get_participants_returns_handles() {
        let mgr = ChatManager::new(10);

        mgr.join(1, "Alice".to_string()).await.unwrap();
        mgr.join(2, "Bob".to_string()).await.unwrap();

        let participants = mgr.get_participants().await;
        assert_eq!(participants.len(), 2);
        assert!(participants.contains(&"Alice".to_string()));
        assert!(participants.contains(&"Bob".to_string()));
    }

    #[tokio::test]
    async fn get_handle_user_id_case_insensitive() {
        let mgr = ChatManager::new(10);

        mgr.join(42, "Alice".to_string()).await.unwrap();

        assert_eq!(mgr.get_handle_user_id("Alice").await, Some(42));
        assert_eq!(mgr.get_handle_user_id("alice").await, Some(42));
        assert_eq!(mgr.get_handle_user_id("ALICE").await, Some(42));
        assert_eq!(mgr.get_handle_user_id("Bob").await, None);
    }

    #[tokio::test]
    async fn broadcast_works_with_subscriber() {
        let mgr = ChatManager::new(10);
        let mut rx = mgr.subscribe();

        mgr.broadcast(ChatMessage::System {
            text: "Test message".to_string(),
        });

        let msg = rx.recv().await.unwrap();
        match msg {
            ChatMessage::System { text } => assert_eq!(text, "Test message"),
            _ => panic!("Expected System message"),
        }
    }

    #[tokio::test]
    async fn broadcast_no_error_without_subscribers() {
        let mgr = ChatManager::new(10);

        // Should not panic or error even with no subscribers
        mgr.broadcast(ChatMessage::System {
            text: "Test".to_string(),
        });
    }
}
