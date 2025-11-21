use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::api::AppState;

/// WebSocket连接信息
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: String,
    pub user_id: Option<String>,
    pub connected_at: Instant,
    pub last_ping: Instant,
}

/// WebSocket连接池
pub type ConnectionPool = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>;

/// 创建新的连接池
pub fn create_connection_pool() -> ConnectionPool {
    Arc::new(RwLock::new(HashMap::new()))
}

/// WebSocket连接处理器
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// 处理WebSocket连接
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let connection_id = Uuid::new_v4().to_string();
    info!("New WebSocket connection: {}", connection_id);

    // 创建消息通道
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // 将连接添加到连接池
    {
        let mut pool = state.ws_pool.write().await;
        pool.insert(connection_id.clone(), tx);
    }

    // 分离socket为发送和接收部分
    let (mut sender, mut receiver) = socket.split();

    // 发送欢迎消息
    let welcome_msg = serde_json::json!({
        "type": "connected",
        "connection_id": connection_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    if let Ok(msg_text) = serde_json::to_string(&welcome_msg) {
        let _ = sender.send(Message::Text(msg_text)).await;
    }

    // 心跳任务
    let connection_id_clone = connection_id.clone();
    let state_clone = state.clone();
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            
            // 检查连接是否还在池中
            let exists = {
                let pool = state_clone.ws_pool.read().await;
                pool.contains_key(&connection_id_clone)
            };
            
            if !exists {
                break;
            }
            
            // 发送ping消息
            let ping_msg = serde_json::json!({
                "type": "ping",
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            
            if let Ok(msg_text) = serde_json::to_string(&ping_msg) {
                let pool = state_clone.ws_pool.read().await;
                if let Some(tx) = pool.get(&connection_id_clone) {
                    let _ = tx.send(Message::Text(msg_text));
                }
            }
        }
    });

    // 接收客户端消息任务
    let connection_id_clone = connection_id.clone();
    let state_clone = state.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    debug!("Received text message from {}: {}", connection_id_clone, text);
                    
                    // 处理客户端消息
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(msg_type) = json.get("type").and_then(|v| v.as_str()) {
                            match msg_type {
                                "pong" => {
                                    debug!("Received pong from {}", connection_id_clone);
                                }
                                "subscribe" => {
                                    // 处理订阅请求
                                    if let Some(topics) = json.get("topics").and_then(|v| v.as_array()) {
                                        info!("Client {} subscribed to topics: {:?}", connection_id_clone, topics);
                                    }
                                }
                                _ => {
                                    warn!("Unknown message type: {}", msg_type);
                                }
                            }
                        }
                    }
                }
                Message::Binary(_) => {
                    debug!("Received binary message from {}", connection_id_clone);
                }
                Message::Ping(data) => {
                    debug!("Received ping from {}", connection_id_clone);
                    let pool = state_clone.ws_pool.read().await;
                    if let Some(tx) = pool.get(&connection_id_clone) {
                        let _ = tx.send(Message::Pong(data));
                    }
                }
                Message::Pong(_) => {
                    debug!("Received pong from {}", connection_id_clone);
                }
                Message::Close(_) => {
                    info!("Client {} requested close", connection_id_clone);
                    break;
                }
            }
        }
    });

    // 发送消息任务
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                error!("Failed to send message to client");
                break;
            }
        }
    });

    // 等待任务完成
    tokio::select! {
        _ = receive_task => {
            debug!("Receive task completed for {}", connection_id);
        }
        _ = send_task => {
            debug!("Send task completed for {}", connection_id);
        }
    }

    // 清理连接
    heartbeat_task.abort();
    {
        let mut pool = state.ws_pool.write().await;
        pool.remove(&connection_id);
    }
    
    info!("WebSocket connection closed: {}", connection_id);
}

/// 向所有连接的客户端广播消息
pub async fn broadcast_message(pool: &ConnectionPool, message: Message) {
    let pool_read = pool.read().await;
    let mut failed_connections = Vec::new();
    
    for (conn_id, tx) in pool_read.iter() {
        if tx.send(message.clone()).is_err() {
            failed_connections.push(conn_id.clone());
        }
    }
    
    drop(pool_read);
    
    // 清理失败的连接
    if !failed_connections.is_empty() {
        let mut pool_write = pool.write().await;
        for conn_id in failed_connections {
            pool_write.remove(&conn_id);
            warn!("Removed failed connection: {}", conn_id);
        }
    }
}

/// 向特定连接发送消息
pub async fn send_to_connection(
    pool: &ConnectionPool,
    connection_id: &str,
    message: Message,
) -> Result<(), String> {
    let pool_read = pool.read().await;
    
    if let Some(tx) = pool_read.get(connection_id) {
        tx.send(message)
            .map_err(|e| format!("Failed to send message: {}", e))?;
        Ok(())
    } else {
        Err(format!("Connection not found: {}", connection_id))
    }
}

/// 获取当前连接数
pub async fn get_connection_count(pool: &ConnectionPool) -> usize {
    let pool_read = pool.read().await;
    pool_read.len()
}

/// 获取所有连接ID
pub async fn get_all_connection_ids(pool: &ConnectionPool) -> Vec<String> {
    let pool_read = pool.read().await;
    pool_read.keys().cloned().collect()
}
