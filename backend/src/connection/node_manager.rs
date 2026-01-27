use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Information about an active node connection.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub user_id: i64,
    pub handle: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

/// Manages active BBS node connections with thread-safe access.
///
/// Tracks which node numbers are in use, enforces the max_nodes limit
/// (artificial scarcity), and provides node assignment/release.
#[derive(Debug, Clone)]
pub struct NodeManager {
    active_nodes: Arc<RwLock<HashMap<usize, NodeInfo>>>,
    max_nodes: usize,
}

impl NodeManager {
    /// Create a new NodeManager with the given capacity.
    pub fn new(max_nodes: usize) -> Self {
        Self {
            active_nodes: Arc::new(RwLock::new(HashMap::new())),
            max_nodes,
        }
    }

    /// Assign the first available node to a user.
    ///
    /// Returns the node number (1-based) or an error if all lines are busy.
    pub async fn assign_node(&self, user_id: i64, handle: String) -> Result<usize, String> {
        let mut nodes = self.active_nodes.write().await;

        if nodes.len() >= self.max_nodes {
            return Err("All lines busy -- please try again later".to_string());
        }

        // Find first available node number (1-based)
        let node_id = (1..=self.max_nodes)
            .find(|id| !nodes.contains_key(id))
            .expect("capacity check passed but no slot found");

        nodes.insert(
            node_id,
            NodeInfo {
                user_id,
                handle,
                connected_at: chrono::Utc::now(),
            },
        );

        Ok(node_id)
    }

    /// Release a node, freeing it for the next caller.
    pub async fn release_node(&self, node_id: usize) {
        let mut nodes = self.active_nodes.write().await;
        nodes.remove(&node_id);
    }

    /// Get current node status: (active_count, max_nodes).
    pub async fn get_status(&self) -> (usize, usize) {
        let nodes = self.active_nodes.read().await;
        (nodes.len(), self.max_nodes)
    }

    /// Get all active nodes as (node_id, handle) pairs, sorted by node_id.
    pub async fn get_active_nodes(&self) -> Vec<(usize, String)> {
        let nodes = self.active_nodes.read().await;
        let mut result: Vec<(usize, String)> = nodes
            .iter()
            .map(|(&id, info)| (id, info.handle.clone()))
            .collect();
        result.sort_by_key(|(id, _)| *id);
        result
    }

    /// Check if a user is already connected on any node.
    pub async fn is_user_connected(&self, user_id: i64) -> bool {
        let nodes = self.active_nodes.read().await;
        nodes.values().any(|info| info.user_id == user_id)
    }

    /// Find the node number for a given user, if connected.
    pub async fn get_node_for_user(&self, user_id: i64) -> Option<usize> {
        let nodes = self.active_nodes.read().await;
        nodes
            .iter()
            .find(|(_, info)| info.user_id == user_id)
            .map(|(&id, _)| id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn assign_node_returns_sequential_ids() {
        let mgr = NodeManager::new(4);

        let n1 = mgr.assign_node(1, "Alice".into()).await.unwrap();
        let n2 = mgr.assign_node(2, "Bob".into()).await.unwrap();
        let n3 = mgr.assign_node(3, "Carol".into()).await.unwrap();

        assert_eq!(n1, 1);
        assert_eq!(n2, 2);
        assert_eq!(n3, 3);
    }

    #[tokio::test]
    async fn assign_node_returns_error_when_full() {
        let mgr = NodeManager::new(2);

        mgr.assign_node(1, "Alice".into()).await.unwrap();
        mgr.assign_node(2, "Bob".into()).await.unwrap();
        let result = mgr.assign_node(3, "Carol".into()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("All lines busy"));
    }

    #[tokio::test]
    async fn release_node_frees_slot() {
        let mgr = NodeManager::new(2);

        mgr.assign_node(1, "Alice".into()).await.unwrap();
        let n2 = mgr.assign_node(2, "Bob".into()).await.unwrap();

        // Full now -- release node 2
        mgr.release_node(n2).await;

        // Should be able to assign again, gets node 2 (first available)
        let n3 = mgr.assign_node(3, "Carol".into()).await.unwrap();
        assert_eq!(n3, 2);
    }

    #[tokio::test]
    async fn get_status_returns_correct_counts() {
        let mgr = NodeManager::new(8);
        assert_eq!(mgr.get_status().await, (0, 8));

        mgr.assign_node(1, "Alice".into()).await.unwrap();
        assert_eq!(mgr.get_status().await, (1, 8));

        mgr.assign_node(2, "Bob".into()).await.unwrap();
        assert_eq!(mgr.get_status().await, (2, 8));

        mgr.release_node(1).await;
        assert_eq!(mgr.get_status().await, (1, 8));
    }

    #[tokio::test]
    async fn get_active_nodes_sorted() {
        let mgr = NodeManager::new(4);

        mgr.assign_node(10, "Charlie".into()).await.unwrap();
        mgr.assign_node(20, "Alice".into()).await.unwrap();

        let active = mgr.get_active_nodes().await;
        assert_eq!(active.len(), 2);
        assert_eq!(active[0], (1, "Charlie".to_string()));
        assert_eq!(active[1], (2, "Alice".to_string()));
    }

    #[tokio::test]
    async fn is_user_connected_works() {
        let mgr = NodeManager::new(4);

        assert!(!mgr.is_user_connected(1).await);

        mgr.assign_node(1, "Alice".into()).await.unwrap();
        assert!(mgr.is_user_connected(1).await);
        assert!(!mgr.is_user_connected(2).await);
    }

    #[tokio::test]
    async fn get_node_for_user_works() {
        let mgr = NodeManager::new(4);

        assert_eq!(mgr.get_node_for_user(1).await, None);

        mgr.assign_node(1, "Alice".into()).await.unwrap();
        assert_eq!(mgr.get_node_for_user(1).await, Some(1));
        assert_eq!(mgr.get_node_for_user(2).await, None);
    }

    #[tokio::test]
    async fn released_node_id_reused() {
        let mgr = NodeManager::new(3);

        let n1 = mgr.assign_node(1, "A".into()).await.unwrap();
        let _n2 = mgr.assign_node(2, "B".into()).await.unwrap();
        let _n3 = mgr.assign_node(3, "C".into()).await.unwrap();

        // Release node 1 (the first)
        mgr.release_node(n1).await;

        // Next assignment should reuse node 1
        let n4 = mgr.assign_node(4, "D".into()).await.unwrap();
        assert_eq!(n4, 1, "should reuse first available node number");
    }
}
