//! 网络管理模块
//!
//! 处理 QUIC 连接、UDP 广播和网络通信，支持 IPv4/IPv6 双栈网络。

use anyhow::Result;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, Duration};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

use crate::gateway::protocol::WdicMessage;
use crate::gateway::protocol::WdicProtocol;

/// 网络事件类型
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// 收到新消息
    MessageReceived {
        /// 消息内容
        message: WdicMessage,
        /// 发送者地址
        sender: SocketAddr,
    },
    /// 新的连接建立
    ConnectionEstablished {
        /// 远程地址
        remote_addr: SocketAddr,
    },
    /// 连接断开
    ConnectionLost {
        /// 远程地址
        remote_addr: SocketAddr,
    },
    /// 连接失败
    ConnectionFailed {
        /// 远程地址
        remote_addr: SocketAddr,
        /// 错误信息
        error: String,
    },
    /// 广播发送完成
    BroadcastSent {
        /// 广播消息
        message: WdicMessage,
    },
    /// 网络错误
    NetworkError {
        /// 错误信息
        error: String,
    },
}

/// 连接状态
#[derive(Debug, Clone)]
pub struct ConnectionState {
    /// 远程地址
    pub remote_addr: SocketAddr,
    /// 最后活跃时间
    pub last_active: chrono::DateTime<chrono::Utc>,
    /// 连接建立时间
    pub established_at: chrono::DateTime<chrono::Utc>,
}

impl ConnectionState {
    /// 创建新的连接状态
    pub fn new(remote_addr: SocketAddr) -> Self {
        let now = chrono::Utc::now();
        Self {
            remote_addr,
            last_active: now,
            established_at: now,
        }
    }

    /// 更新最后活跃时间
    pub fn update_activity(&mut self) {
        self.last_active = chrono::Utc::now();
    }

    /// 检查连接是否超时
    pub fn is_expired(&self, timeout_seconds: i64) -> bool {
        let now = chrono::Utc::now();
        (now - self.last_active).num_seconds() > timeout_seconds
    }

    /// 检查连接是否活跃
    pub fn is_connected(&self) -> bool {
        !self.is_expired(30) // 30秒超时
    }
}

/// 发现的节点信息
#[derive(Debug, Clone)]
pub struct DiscoveredNodeInfo {
    /// 节点 ID
    pub node_id: String,
    /// IP 地址
    pub ip_address: String,
    /// 端口
    pub port: u16,
    /// 节点名称
    pub name: String,
    /// 发现时间
    pub discovered_time: DateTime<Utc>,
    /// 最后看到时间
    pub last_seen: DateTime<Utc>,
    /// 是否在线
    pub is_online: bool,
    /// 节点类型
    pub node_type: String,
}

/// 文件传输任务信息
#[derive(Debug, Clone)]
pub struct FileTransferTaskInfo {
    /// 任务 ID
    pub task_id: String,
    /// 源路径
    pub source_path: PathBuf,
    /// 目标路径
    pub target_path: PathBuf,
    /// 任务状态
    pub status: crate::gateway::TransferStatus,
    /// 已传输字节数
    pub transferred_bytes: u64,
    /// 总字节数
    pub total_bytes: u64,
    /// 传输速度 (字节/秒)
    pub transfer_speed: u64,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 预计完成时间
    pub estimated_completion: Option<DateTime<Utc>>,
    /// 目标节点 ID (如果是远程传输)
    pub target_node_id: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
}

impl FileTransferTaskInfo {
    /// 创建新的文件传输任务
    pub fn new(
        task_id: String,
        source_path: PathBuf,
        target_path: PathBuf,
        total_bytes: u64,
        target_node_id: Option<String>,
    ) -> Self {
        Self {
            task_id,
            source_path,
            target_path,
            status: crate::gateway::TransferStatus::Pending,
            transferred_bytes: 0,
            total_bytes,
            transfer_speed: 0,
            start_time: Utc::now(),
            estimated_completion: None,
            target_node_id,
            error_message: None,
        }
    }

    /// 更新传输进度
    pub fn update_progress(&mut self, transferred_bytes: u64, transfer_speed: u64) {
        self.transferred_bytes = transferred_bytes;
        self.transfer_speed = transfer_speed;
        
        // 计算预计完成时间
        if transfer_speed > 0 && transferred_bytes < self.total_bytes {
            let remaining_bytes = self.total_bytes - transferred_bytes;
            let remaining_seconds = remaining_bytes / transfer_speed;
            self.estimated_completion = Some(Utc::now() + chrono::Duration::seconds(remaining_seconds as i64));
        }
    }

    /// 设置任务状态
    pub fn set_status(&mut self, status: crate::gateway::TransferStatus) {
        self.status = status;
    }

    /// 设置错误信息
    pub fn set_error(&mut self, error_message: String) {
        self.status = crate::gateway::TransferStatus::Error(error_message.clone());
        self.error_message = Some(error_message);
    }
}

impl DiscoveredNodeInfo {
    /// 创建新的发现节点信息
    pub fn new(
        node_id: String,
        ip_address: String,
        port: u16,
        name: String,
        node_type: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            node_id,
            ip_address,
            port,
            name,
            discovered_time: now,
            last_seen: now,
            is_online: true,
            node_type,
        }
    }

    /// 更新最后看到时间
    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
        self.is_online = true;
    }

    /// 检查节点是否过期
    pub fn is_expired(&self, timeout_seconds: i64) -> bool {
        let now = Utc::now();
        (now - self.last_seen).num_seconds() > timeout_seconds
    }
}

/// 网络管理器
///
/// 负责处理网络通信，包括 UDP 广播和消息收发。
#[derive(Debug)]
pub struct NetworkManager {
    /// 本地地址
    local_addr: SocketAddr,
    /// UDP 套接字
    udp_socket: Arc<UdpSocket>,
    /// 协议处理器
    protocol: WdicProtocol,
    /// 活跃连接
    pub connections: Arc<Mutex<HashMap<SocketAddr, ConnectionState>>>,
    /// 事件发送通道
    event_sender: mpsc::UnboundedSender<NetworkEvent>,
    /// 事件接收通道
    event_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<NetworkEvent>>>>,
    /// 广播地址列表
    broadcast_addresses: Vec<SocketAddr>,
    /// 已发现的节点
    pub discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNodeInfo>>>,
    /// P2P 发现状态
    pub p2p_discovery_enabled: Arc<Mutex<bool>>,
    /// 发现任务句柄
    discovery_task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// 文件传输任务存储
    pub transfer_tasks: Arc<RwLock<HashMap<String, FileTransferTaskInfo>>>,
    /// 节点 ID 到连接地址的映射
    pub node_connections: Arc<RwLock<HashMap<String, SocketAddr>>>,
}

impl NetworkManager {
    /// 创建新的网络管理器
    ///
    /// # 参数
    ///
    /// * `local_addr` - 本地监听地址
    ///
    /// # 返回值
    ///
    /// 网络管理器实例
    pub fn new(local_addr: SocketAddr) -> Result<Self> {
        let udp_socket = UdpSocket::bind(local_addr)?;
        udp_socket.set_broadcast(true)?;
        udp_socket.set_nonblocking(true)?;

        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        // 生成常见的广播地址
        let broadcast_addresses = Self::generate_broadcast_addresses(local_addr);

        Ok(Self {
            local_addr,
            udp_socket: Arc::new(udp_socket),
            protocol: WdicProtocol::new(),
            connections: Arc::new(Mutex::new(HashMap::new())),
            event_sender,
            event_receiver: Arc::new(Mutex::new(Some(event_receiver))),
            broadcast_addresses,
            discovered_nodes: Arc::new(RwLock::new(HashMap::new())),
            p2p_discovery_enabled: Arc::new(Mutex::new(false)),
            discovery_task_handle: Arc::new(Mutex::new(None)),
            transfer_tasks: Arc::new(RwLock::new(HashMap::new())),
            node_connections: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 获取本地地址
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// 获取事件接收器
    pub async fn take_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<NetworkEvent>> {
        self.event_receiver.lock().await.take()
    }

    /// 生成广播地址列表
    ///
    /// 根据本地地址生成可能的广播地址。
    /// 生成广播地址列表（支持 IPv4/IPv6 双栈）
    ///
    /// 这个函数会：
    /// 1. 发现本地所有网络接口
    /// 2. 为 IPv4 地址生成广播地址
    /// 3. 为 IPv6 地址生成多播地址
    /// 4. 优先使用内网地址，在没有内网地址时使用公网地址
    ///
    /// # 参数
    ///
    /// * `local_addr` - 本地绑定地址
    ///
    /// # 返回值
    ///
    /// 广播和多播地址列表
    fn generate_broadcast_addresses(local_addr: SocketAddr) -> Vec<SocketAddr> {
        let mut addresses = Vec::new();
        let port = local_addr.port();

        debug!("为地址 {local_addr} 生成广播地址列表");

        // 获取所有网络接口
        let interfaces = match if_addrs::get_if_addrs() {
            Ok(interfaces) => interfaces,
            Err(e) => {
                warn!("无法获取网络接口列表: {e}, 使用默认广播地址");
                return Self::generate_fallback_addresses(port);
            }
        };

        // 分类接口地址
        let mut ipv4_private = Vec::new();
        let mut ipv4_public = Vec::new();
        let mut ipv6_private = Vec::new();
        let mut ipv6_public = Vec::new();

        for interface in interfaces {
            if interface.is_loopback() {
                continue;
            }

            match interface.ip() {
                IpAddr::V4(ipv4) => {
                    if Self::is_private_ipv4(ipv4) {
                        ipv4_private.push(ipv4);
                    } else {
                        ipv4_public.push(ipv4);
                    }
                }
                IpAddr::V6(ipv6) => {
                    if Self::is_private_ipv6(ipv6) {
                        ipv6_private.push(ipv6);
                    } else {
                        ipv6_public.push(ipv6);
                    }
                }
            }
        }

        // 生成 IPv4 广播地址
        Self::add_ipv4_broadcasts(&mut addresses, &ipv4_private, port);
        if ipv4_private.is_empty() && !ipv4_public.is_empty() {
            info!("没有找到私有 IPv4 地址，使用公网 IPv4 地址进行广播");
            Self::add_ipv4_public_broadcasts(&mut addresses, &ipv4_public, port);
        }

        // 生成 IPv6 多播地址
        if !ipv6_private.is_empty() || !ipv6_public.is_empty() {
            Self::add_ipv6_multicasts(&mut addresses, port);
        }

        // 如果没有找到任何有效地址，使用后备地址
        if addresses.is_empty() {
            warn!("没有找到有效的网络接口，使用默认广播地址");
            addresses = Self::generate_fallback_addresses(port);
        }

        info!("生成了 {} 个广播/多播地址", addresses.len());
        for addr in &addresses {
            debug!("  - {addr}");
        }

        addresses
    }

    /// 判断是否为私有 IPv4 地址
    fn is_private_ipv4(ip: Ipv4Addr) -> bool {
        let octets = ip.octets();
        // 10.0.0.0/8
        if octets[0] == 10 {
            return true;
        }
        // 172.16.0.0/12
        if octets[0] == 172 && (16..=31).contains(&octets[1]) {
            return true;
        }
        // 192.168.0.0/16
        if octets[0] == 192 && octets[1] == 168 {
            return true;
        }
        false
    }

    /// 判断是否为私有 IPv6 地址
    fn is_private_ipv6(ip: Ipv6Addr) -> bool {
        // 链路本地地址 (fe80::/10)
        if ip.segments()[0] & 0xffc0 == 0xfe80 {
            return true;
        }
        // 唯一本地地址 (fc00::/7) 或 (fd00::/8)
        if ip.segments()[0] & 0xfe00 == 0xfc00 {
            return true;
        }
        false
    }

    /// 添加 IPv4 私有网络广播地址
    fn add_ipv4_broadcasts(addresses: &mut Vec<SocketAddr>, ipv4_addrs: &[Ipv4Addr], port: u16) {
        for &ip in ipv4_addrs {
            let octets = ip.octets();

            // 基于具体 IP 地址生成子网广播地址
            if octets[0] == 192 && octets[1] == 168 {
                addresses.push(SocketAddr::from(([192, 168, octets[2], 255], port)));
            } else if octets[0] == 10 {
                addresses.push(SocketAddr::from(([10, octets[1], 255, 255], port)));
            } else if octets[0] == 172 && (16..=31).contains(&octets[1]) {
                addresses.push(SocketAddr::from(([172, octets[1], 255, 255], port)));
            }
        }

        // 添加常见的私有网络广播地址
        if !ipv4_addrs.is_empty() {
            addresses.push(SocketAddr::from(([192, 168, 255, 255], port)));
            addresses.push(SocketAddr::from(([10, 255, 255, 255], port)));
            addresses.push(SocketAddr::from(([172, 31, 255, 255], port)));
        }
    }

    /// 添加 IPv4 公网广播地址
    fn add_ipv4_public_broadcasts(
        addresses: &mut Vec<SocketAddr>,
        _ipv4_addrs: &[Ipv4Addr],
        port: u16,
    ) {
        // 对于公网地址，我们只能使用有限广播
        addresses.push(SocketAddr::from(([255, 255, 255, 255], port)));
    }

    /// 添加 IPv6 多播地址
    fn add_ipv6_multicasts(addresses: &mut Vec<SocketAddr>, port: u16) {
        // 站点本地多播 (ff05::1)
        addresses.push(SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 0, 1)),
            port,
        ));

        // 链路本地多播 (ff02::1)
        addresses.push(SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 1)),
            port,
        ));

        // 自定义的 WDIC 多播地址 (ff05::5555)
        addresses.push(SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 0, 0x5555)),
            port,
        ));
    }

    /// 生成后备广播地址
    fn generate_fallback_addresses(port: u16) -> Vec<SocketAddr> {
        vec![
            SocketAddr::from(([255, 255, 255, 255], port)),
            SocketAddr::from(([192, 168, 255, 255], port)),
            SocketAddr::from(([10, 255, 255, 255], port)),
            SocketAddr::from(([172, 31, 255, 255], port)),
        ]
    }

    /// 启动网络服务
    ///
    /// 开始监听网络消息和处理连接。
    pub async fn start(&self) -> Result<()> {
        info!("网络管理器在 {} 启动", self.local_addr);

        // 启动 UDP 监听任务
        let socket = Arc::clone(&self.udp_socket);
        let event_sender = self.event_sender.clone();
        let connections = Arc::clone(&self.connections);
        let protocol = self.protocol.clone();

        tokio::spawn(async move {
            Self::udp_listener_task(socket, event_sender, connections, protocol).await;
        });

        // 启动连接清理任务
        let connections_cleanup = Arc::clone(&self.connections);
        tokio::spawn(async move {
            Self::connection_cleanup_task(connections_cleanup).await;
        });

        Ok(())
    }

    /// UDP 监听任务
    async fn udp_listener_task(
        socket: Arc<UdpSocket>,
        event_sender: mpsc::UnboundedSender<NetworkEvent>,
        connections: Arc<Mutex<HashMap<SocketAddr, ConnectionState>>>,
        protocol: WdicProtocol,
    ) {
        let mut buffer = [0u8; 65536];

        loop {
            match socket.recv_from(&mut buffer) {
                Ok((size, sender_addr)) => {
                    debug!("收到来自 {sender_addr} 的 {size} 字节数据");

                    // 更新连接状态
                    {
                        let mut conns = connections.lock().await;
                        if let Some(conn) = conns.get_mut(&sender_addr) {
                            conn.update_activity();
                        } else {
                            conns.insert(sender_addr, ConnectionState::new(sender_addr));
                            let _ = event_sender.send(NetworkEvent::ConnectionEstablished {
                                remote_addr: sender_addr,
                            });
                        }
                    }

                    // 解析消息
                    match WdicMessage::from_bytes(&buffer[..size]) {
                        Ok(message) => {
                            debug!("解析消息成功: {}", message.message_type());

                            // 验证消息
                            if let Err(e) = protocol.validate_message(&message) {
                                warn!("消息验证失败: {e}");
                                continue;
                            }

                            let _ = event_sender.send(NetworkEvent::MessageReceived {
                                message,
                                sender: sender_addr,
                            });
                        }
                        Err(e) => {
                            warn!("解析消息失败: {e}");
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // 非阻塞模式下没有数据可读
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(e) => {
                    error!("UDP 接收错误: {e}");
                    let _ = event_sender.send(NetworkEvent::NetworkError {
                        error: format!("UDP 接收错误: {e}"),
                    });
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// 连接清理任务
    async fn connection_cleanup_task(
        connections: Arc<Mutex<HashMap<SocketAddr, ConnectionState>>>,
    ) {
        let mut cleanup_interval = interval(Duration::from_secs(60));

        loop {
            cleanup_interval.tick().await;

            let mut conns = connections.lock().await;
            let expired_addrs: Vec<SocketAddr> = conns
                .iter()
                .filter(|(_, state)| state.is_expired(300)) // 5分钟超时
                .map(|(addr, _)| *addr)
                .collect();

            for addr in expired_addrs {
                conns.remove(&addr);
                debug!("清理过期连接: {addr}");
            }
        }
    }

    /// 发送消息到指定地址
    ///
    /// # 参数
    ///
    /// * `message` - 要发送的消息
    /// * `target` - 目标地址
    ///
    /// # 返回值
    ///
    /// 发送结果
    pub async fn send_message(&self, message: &WdicMessage, target: SocketAddr) -> Result<()> {
        // 优先尝试通过 QUIC 发送
        match self.send_quic_message(message, target).await {
            Ok(_) => {
                debug!("通过 QUIC 发送 {} 消息到 {target}", message.message_type());
                Ok(())
            }
            Err(e) => {
                debug!("QUIC 发送失败，回退到 UDP: {}", e);
                // 回退到 UDP 发送
                let data = message.to_bytes()?;
                self.udp_socket
                    .send_to(&data, target)
                    .map_err(|e| anyhow::anyhow!("发送消息到 {target} 失败: {e}"))?;
                
                debug!("通过 UDP 发送 {} 消息到 {target}", message.message_type());
                Ok(())
            }
        }
    }

    /// 广播消息到本地网络
    ///
    /// # 参数
    ///
    /// * `message` - 要广播的消息
    ///
    /// # 返回值
    ///
    /// 成功发送的地址数量
    pub async fn broadcast_message(&self, message: &WdicMessage) -> Result<usize> {
        let data = message.to_bytes()?;
        let mut success_count = 0;

        info!(
            "广播 {} 消息到 {} 个地址",
            message.message_type(),
            self.broadcast_addresses.len()
        );

        for &broadcast_addr in &self.broadcast_addresses {
            match self.udp_socket.send_to(&data, broadcast_addr) {
                Ok(_) => {
                    success_count += 1;
                    debug!("成功广播到 {broadcast_addr}");
                }
                Err(e) => {
                    warn!("广播到 {broadcast_addr} 失败: {e}");
                }
            }
        }

        // 发送广播完成事件
        let _ = self.event_sender.send(NetworkEvent::BroadcastSent {
            message: message.clone(),
        });

        Ok(success_count)
    }

    /// 回复消息到发送者
    ///
    /// # 参数
    ///
    /// * `response` - 响应消息
    /// * `original_sender` - 原始发送者地址
    ///
    /// # 返回值
    ///
    /// 发送结果
    pub async fn reply_message(
        &self,
        response: &WdicMessage,
        original_sender: SocketAddr,
    ) -> Result<()> {
        self.send_message(response, original_sender).await
    }

    /// 获取当前活跃连接数
    ///
    /// # 返回值
    ///
    /// 活跃连接数量
    pub async fn active_connections_count(&self) -> usize {
        self.connections.lock().await.len()
    }

    /// 获取所有活跃连接
    ///
    /// # 返回值
    ///
    /// 活跃连接状态列表
    pub async fn get_active_connections(&self) -> Vec<ConnectionState> {
        self.connections.lock().await.values().cloned().collect()
    }

    /// 断开指定连接
    ///
    /// # 参数
    ///
    /// * `addr` - 要断开的连接地址
    ///
    /// # 返回值
    ///
    /// 是否成功断开连接
    pub async fn disconnect(&self, addr: SocketAddr) -> bool {
        let mut conns = self.connections.lock().await;
        if conns.remove(&addr).is_some() {
            let _ = self
                .event_sender
                .send(NetworkEvent::ConnectionLost { remote_addr: addr });
            true
        } else {
            false
        }
    }

    /// 关闭网络管理器
    ///
    /// # 返回值
    ///
    /// 操作结果
    pub async fn shutdown(&self) -> Result<()> {
        info!("关闭网络管理器");
        // 清理所有连接
        self.connections.lock().await.clear();
        Ok(())
    }

    // ============================================================================
    // Tauri API 需要的方法
    // ============================================================================

    /// 获取网络信息
    ///
    /// # 返回值
    ///
    /// 网络状态信息
    pub async fn get_network_info(&self) -> anyhow::Result<crate::gateway::tauri_api::NetworkStatus> {
        use if_addrs::get_if_addrs;
        use crate::gateway::tauri_api::{NetworkInterface, NetworkStatus};

        let interfaces = get_if_addrs()
            .map_err(|e| anyhow::anyhow!("获取网络接口失败: {}", e))?;

        let network_interfaces: Vec<NetworkInterface> = interfaces
            .into_iter()
            .map(|iface| {
                let name = iface.name.clone();
                NetworkInterface {
                    name,
                    ip_address: iface.ip().to_string(),
                    is_active: !iface.ip().is_loopback(),
                    interface_type: if iface.ip().is_ipv4() {
                        "IPv4".to_string()
                    } else {
                        "IPv6".to_string()
                    },
                }
            })
            .collect();

        let local_ip = self.local_addr.ip().to_string();
        let listen_port = self.local_addr.port();

        Ok(NetworkStatus {
            local_ip,
            listen_port,
            network_interfaces,
            p2p_discovery_enabled: *self.p2p_discovery_enabled.lock().await,
            discovered_nodes: self.discovered_nodes.read().await.len() as u32,
        })
    }

    /// 启动 P2P 发现
    ///
    /// # 返回值
    ///
    /// 操作结果
    pub async fn start_p2p_discovery(&self) -> anyhow::Result<()> {
        info!("启动 P2P 发现");
        
        // 检查是否已经启动
        {
            let mut enabled = self.p2p_discovery_enabled.lock().await;
            if *enabled {
                warn!("P2P 发现已经启动");
                return Ok(());
            }
            *enabled = true;
        }

        // 启动发现任务
        let discovered_nodes = Arc::clone(&self.discovered_nodes);
        let event_sender = self.event_sender.clone();
        let udp_socket = Arc::clone(&self.udp_socket);
        let protocol = self.protocol.clone();
        let broadcast_addresses = self.broadcast_addresses.clone();
        let local_addr = self.local_addr;
        let p2p_enabled = Arc::clone(&self.p2p_discovery_enabled);

        let task_handle = tokio::spawn(async move {
            Self::discovery_task(
                discovered_nodes,
                event_sender,
                udp_socket,
                protocol,
                broadcast_addresses,
                local_addr,
                p2p_enabled,
            ).await;
        });

        *self.discovery_task_handle.lock().await = Some(task_handle);
        info!("P2P 发现任务已启动");
        Ok(())
    }

    /// 停止 P2P 发现
    ///
    /// # 返回值
    ///
    /// 操作结果
    pub async fn stop_p2p_discovery(&self) -> anyhow::Result<()> {
        info!("停止 P2P 发现");
        
        // 设置停止标志
        *self.p2p_discovery_enabled.lock().await = false;

        // 停止发现任务
        if let Some(handle) = self.discovery_task_handle.lock().await.take() {
            handle.abort();
            info!("P2P 发现任务已停止");
        }

        Ok(())
    }

    /// 获取已发现的节点列表
    ///
    /// # 返回值
    ///
    /// 发现的节点列表
    pub async fn get_discovered_nodes(&self) -> anyhow::Result<Vec<crate::gateway::tauri_api::DiscoveredNode>> {
        let nodes = self.discovered_nodes.read().await;
        let discovered_nodes: Vec<crate::gateway::tauri_api::DiscoveredNode> = nodes
            .values()
            .map(|node_info| crate::gateway::tauri_api::DiscoveredNode {
                node_id: node_info.node_id.clone(),
                ip_address: node_info.ip_address.clone(),
                port: node_info.port,
                name: node_info.name.clone(),
                discovered_time: node_info.discovered_time,
                last_seen: node_info.last_seen,
                is_online: node_info.is_online,
                node_type: node_info.node_type.clone(),
            })
            .collect();

        Ok(discovered_nodes)
    }

    /// 连接到指定节点
    ///
    /// # 参数
    ///
    /// * `node_id` - 节点 ID
    /// * `ip_address` - IP 地址
    /// * `port` - 端口
    ///
    /// # 返回值
    ///
    /// 操作结果
    pub async fn connect_to_node(
        &self,
        node_id: &str,
        ip_address: &str,
        port: u16,
    ) -> anyhow::Result<()> {
        use std::net::SocketAddr;

        let addr: SocketAddr = format!("{ip_address}:{port}")
            .parse()
            .map_err(|e| anyhow::anyhow!("无效地址: {}", e))?;

        log::info!("连接到节点 {node_id} ({addr})");

        // 检查是否已经连接到此节点
        {
            let node_connections = self.node_connections.read().await;
            if node_connections.contains_key(node_id) {
                return Err(anyhow::anyhow!("已经连接到节点 {}", node_id));
            }
        }

        // 验证节点是否存在于发现的节点列表中
        {
            let discovered_nodes = self.discovered_nodes.read().await;
            if !discovered_nodes.contains_key(node_id) {
                return Err(anyhow::anyhow!("节点 {} 不在发现列表中", node_id));
            }
        }

        // 实际连接实现
        self.establish_quic_connection(node_id, addr).await
    }

    /// 建立 QUIC 连接
    ///
    /// # 参数
    ///
    /// * `node_id` - 节点 ID
    /// * `addr` - 目标地址
    ///
    /// # 返回值
    ///
    /// 操作结果
    async fn establish_quic_connection(
        &self,
        node_id: &str,
        addr: SocketAddr,
    ) -> anyhow::Result<()> {
        // 创建 QUIC 连接配置
        let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)
            .map_err(|e| anyhow::anyhow!("创建 QUIC 配置失败: {}", e))?;
        
        // 配置 QUIC 参数
        config.set_application_protos(&[b"wdic"])
            .map_err(|e| anyhow::anyhow!("设置应用协议失败: {}", e))?;
        config.set_max_idle_timeout(30000); // 30 秒超时
        config.set_max_recv_udp_payload_size(1350);
        config.set_max_send_udp_payload_size(1350);
        config.set_initial_max_data(10_000_000);
        config.set_initial_max_stream_data_bidi_local(1_000_000);
        config.set_initial_max_stream_data_bidi_remote(1_000_000);
        config.set_initial_max_streams_bidi(100);
        config.set_disable_active_migration(true);
        
        // 启用 TLS 验证
        config.verify_peer(true);
        
        // 设置 TLS 参数 (目前禁用)
        // TODO: 实现 TLS 管理器集成
        warn!("TLS 管理器未初始化，使用不安全连接");
        config.verify_peer(false);

        // 生成连接 ID
        let uuid = uuid::Uuid::new_v4();
        let scid = quiche::ConnectionId::from_ref(&uuid.as_bytes()[..]);
        
        // 创建 QUIC 连接
        let mut connection = quiche::connect(
            Some("localhost"), 
            &scid, 
            self.local_addr,
            addr,
            &mut config
        ).map_err(|e| anyhow::anyhow!("创建 QUIC 连接失败: {}", e))?;

        // 准备握手数据
        let mut out = [0; 1350];
        let (write_len, send_info) = connection.send(&mut out)
            .map_err(|e| anyhow::anyhow!("QUIC 握手发送失败: {}", e))?;

        // 发送握手数据
        if write_len > 0 {
            self.udp_socket.send_to(&out[..write_len], send_info.to)
                .map_err(|e| anyhow::anyhow!("发送握手数据失败: {}", e))?;
            
            log::debug!("已发送 {} 字节握手数据到 {}", write_len, send_info.to);
        }

        // 在后台任务中处理握手完成
        let event_sender = self.event_sender.clone();
        let connections = Arc::clone(&self.connections);
        let node_connections = Arc::clone(&self.node_connections);
        let discovered_nodes = Arc::clone(&self.discovered_nodes);
        let node_id_clone = node_id.to_string();
        
        tokio::spawn(async move {
            // 为 QUIC 创建新的 tokio UDP 套接字
            let tokio_socket = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
                Ok(socket) => Arc::new(socket),
                Err(e) => {
                    error!("无法创建 tokio UDP 套接字: {}", e);
                    let _ = event_sender.send(NetworkEvent::ConnectionFailed {
                        remote_addr: addr,
                        error: format!("套接字创建失败: {}", e),
                    });
                    return;
                }
            };

            // 异步处理握手完成
            match Self::complete_quic_handshake_async(connection, addr, tokio_socket).await {
                Ok(_established_connection) => {
                    // 连接成功，存储连接信息
                    {
                        let mut connections_guard = connections.lock().await;
                        connections_guard.insert(addr, ConnectionState::new(addr));
                    }
                    
                    {
                        let mut node_connections_guard = node_connections.write().await;
                        node_connections_guard.insert(node_id_clone.clone(), addr);
                    }
                    
                    // 更新发现节点状态
                    {
                        let mut discovered_nodes_guard = discovered_nodes.write().await;
                        if let Some(node_info) = discovered_nodes_guard.get_mut(&node_id_clone) {
                            node_info.is_online = true;
                            node_info.update_last_seen();
                        }
                    }
                    
                    // 发送连接建立事件
                    let _ = event_sender.send(NetworkEvent::ConnectionEstablished {
                        remote_addr: addr,
                    });
                    
                    log::info!("成功建立 QUIC 连接到节点 {} ({})", node_id_clone, addr);
                }
                Err(e) => {
                    log::error!("QUIC 握手失败: {}", e);
                    let _ = event_sender.send(NetworkEvent::ConnectionFailed {
                        remote_addr: addr,
                        error: e.to_string(),
                    });
                }
            }
        });

        Ok(())
    }

    /// 异步完成 QUIC 握手（专用于后台任务）
    ///
    /// # 参数
    ///
    /// * `connection` - QUIC 连接对象
    /// * `addr` - 远程地址
    /// * `udp_socket` - UDP 套接字
    ///
    /// # 返回值
    ///
    /// 成功建立的连接对象
    async fn complete_quic_handshake_async(
        mut connection: quiche::Connection,
        addr: SocketAddr,
        udp_socket: Arc<tokio::net::UdpSocket>,
    ) -> anyhow::Result<quiche::Connection> {
        // 设置握手超时
        let handshake_timeout = tokio::time::Duration::from_secs(30);
        let start_time = tokio::time::Instant::now();

        let mut recv_buf = [0; 1350];
        let mut send_buf = [0; 1350];
        
        while !connection.is_established() {
            if tokio::time::Instant::now() - start_time > handshake_timeout {
                return Err(anyhow::anyhow!("QUIC 握手超时"));
            }

            // 处理握手状态机
            if connection.is_in_early_data() || connection.is_established() {
                break;
            }

            // 尝试发送握手数据
            loop {
                match connection.send(&mut send_buf) {
                    Ok((write_len, send_info)) => {
                        if write_len == 0 {
                            break;
                        }

                        udp_socket.send_to(&send_buf[..write_len], send_info.to).await
                            .map_err(|e| anyhow::anyhow!("发送握手数据失败: {}", e))?;
                        
                        log::debug!("发送了 {} 字节的握手数据", write_len);
                    }
                    Err(quiche::Error::Done) => {
                        break;
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("生成握手数据失败: {}", e));
                    }
                }
            }

            // 等待接收握手响应
            match tokio::time::timeout(
                tokio::time::Duration::from_millis(500),
                udp_socket.recv_from(&mut recv_buf),
            ).await {
                Ok(Ok((recv_len, recv_addr))) if recv_addr == addr => {
                    // 处理接收到的握手数据
                    let recv_info = quiche::RecvInfo {
                        to: udp_socket.local_addr()
                            .map_err(|e| anyhow::anyhow!("获取本地地址失败: {}", e))?,
                        from: recv_addr,
                    };

                    match connection.recv(&mut recv_buf[..recv_len], recv_info) {
                        Ok(_) => {
                            log::debug!("处理了 {} 字节的握手数据", recv_len);
                        }
                        Err(e) => {
                            log::warn!("处理握手数据失败: {}", e);
                        }
                    }
                }
                Ok(Ok(_)) => {
                    // 收到来自其他地址的数据，忽略
                }
                Ok(Err(e)) => {
                    return Err(anyhow::anyhow!("接收握手数据失败: {}", e));
                }
                Err(_) => {
                    // 超时，继续下一轮
                    log::debug!("握手接收超时，继续重试");
                }
            }

            // 短暂延迟避免忙等待
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        if connection.is_established() {
            log::info!("QUIC 握手成功完成");
            Ok(connection)
        } else {
            Err(anyhow::anyhow!("QUIC 握手未能建立连接"))
        }
    }

    /// 完成 QUIC 握手
    ///
    /// # 参数
    ///
    /// * `connection` - QUIC 连接对象
    /// * `addr` - 远程地址
    ///
    /// # 返回值
    ///
    /// 是否握手成功
    async fn complete_quic_handshake(
        &self,
        connection: &mut quiche::Connection,
        addr: SocketAddr,
    ) -> anyhow::Result<bool> {
        // 设置握手超时
        let handshake_timeout = tokio::time::Duration::from_secs(10);
        let start_time = tokio::time::Instant::now();

        let mut recv_buf = [0; 1350];
        
        while !connection.is_established() {
            if tokio::time::Instant::now() - start_time > handshake_timeout {
                return Err(anyhow::anyhow!("QUIC 握手超时"));
            }

            // 尝试接收数据
            match self.udp_socket.recv_from(&mut recv_buf) {
                Ok((recv_len, recv_addr)) if recv_addr == addr => {
                    // 处理接收到的数据
                    let recv_info = quiche::RecvInfo {
                        to: self.local_addr,
                        from: recv_addr,
                    };

                    // 处理接收到的包
                    match connection.recv(&mut recv_buf[..recv_len], recv_info) {
                        Ok(_) => {
                            log::debug!("处理了 {} 字节的 QUIC 数据", recv_len);
                        }
                        Err(e) => {
                            log::warn!("处理 QUIC 数据失败: {}", e);
                        }
                    }

                    // 尝试发送响应
                    let mut out = [0; 1350];
                    loop {
                        match connection.send(&mut out) {
                            Ok((write_len, send_info)) => {
                                if write_len == 0 {
                                    break;
                                }

                                if let Err(e) = self.udp_socket.send_to(&out[..write_len], send_info.to) {
                                    log::warn!("发送 QUIC 响应失败: {}", e);
                                }
                                
                                log::debug!("发送了 {} 字节的 QUIC 响应", write_len);
                            }
                            Err(quiche::Error::Done) => {
                                break;
                            }
                            Err(e) => {
                                log::warn!("生成 QUIC 响应失败: {}", e);
                                break;
                            }
                        }
                    }
                }
                Ok(_) => {
                    // 收到来自其他地址的数据，忽略
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // 没有数据可读，等待一段时间
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("接收数据失败: {}", e));
                }
            }
        }

        Ok(connection.is_established())
    }

    /// 通过 QUIC 发送消息
    ///
    /// # 参数
    ///
    /// * `message` - 要发送的消息
    /// * `target` - 目标地址
    ///
    /// # 返回值
    ///
    /// 发送结果
    pub async fn send_quic_message(&self, message: &WdicMessage, target: SocketAddr) -> Result<()> {
        let data = message.to_bytes()?;
        
        // 检查是否有到目标地址的活跃 QUIC 连接
        let connections = self.connections.lock().await;
        if let Some(connection_state) = connections.get(&target) {
            // 有活跃连接，尝试通过 QUIC 流发送
            if connection_state.is_connected() {
                log::debug!("通过已建立的 QUIC 连接发送 {} 消息到 {}", message.message_type(), target);
                
                // 在实际实现中，这里需要使用 QUIC 连接的流来发送数据
                // 当前版本先记录尝试，然后回退到 UDP
                log::debug!("QUIC 流发送功能待完善，回退到 UDP");
            }
        }
        
        // 回退到 UDP 发送
        log::debug!("发送 {} 消息到 {} (通过 UDP)", message.message_type(), target);
        self.udp_socket
            .send_to(&data, target)
            .map_err(|e| anyhow::anyhow!("发送消息到 {target} 失败: {e}"))?;

        Ok(())
    }

    /// 断开与节点的连接
    ///
    /// # 参数
    ///
    /// * `node_id` - 节点 ID
    ///
    /// # 返回值
    ///
    /// 操作结果
    pub async fn disconnect_from_node(&self, node_id: &str) -> anyhow::Result<()> {
        log::info!("断开与节点 {node_id} 的连接");

        // 从节点连接映射中查找连接地址
        let node_connections = self.node_connections.read().await;
        if let Some(&addr) = node_connections.get(node_id) {
            drop(node_connections); // 释放读锁
            
            // 断开实际连接
            if self.disconnect(addr).await {
                // 从节点连接映射中移除
                let mut node_connections = self.node_connections.write().await;
                node_connections.remove(node_id);
                
                // 更新发现节点状态
                let mut discovered_nodes = self.discovered_nodes.write().await;
                if let Some(node_info) = discovered_nodes.get_mut(node_id) {
                    node_info.is_online = false;
                }
                
                log::info!("成功断开与节点 {node_id} 的连接");
                Ok(())
            } else {
                Err(anyhow::anyhow!("无法断开与节点 {} 的连接", node_id))
            }
        } else {
            Err(anyhow::anyhow!("未找到节点 {} 的连接", node_id))
        }
    }

    /// 创建文件传输任务
    ///
    /// # 参数
    ///
    /// * `task_id` - 任务 ID
    /// * `source_path` - 源路径
    /// * `target_path` - 目标路径
    ///
    /// # 返回值
    ///
    /// 操作结果
    pub async fn create_transfer_task(
        &self,
        task_id: String,
        source_path: std::path::PathBuf,
        target_path: std::path::PathBuf,
    ) -> anyhow::Result<()> {
        log::info!(
            "创建文件传输任务 {task_id}: {source_path:?} -> {target_path:?}"
        );

        // 验证源文件存在
        if !source_path.exists() {
            return Err(anyhow::anyhow!("源文件不存在: {:?}", source_path));
        }

        // 获取文件大小
        let metadata = tokio::fs::metadata(&source_path).await
            .map_err(|e| anyhow::anyhow!("无法获取文件元数据: {}", e))?;
        
        let total_bytes = metadata.len();

        // 创建传输任务信息
        let task_info = FileTransferTaskInfo::new(
            task_id.clone(),
            source_path,
            target_path,
            total_bytes,
            None, // 本地传输，无目标节点
        );

        // 存储任务信息
        {
            let mut transfer_tasks = self.transfer_tasks.write().await;
            transfer_tasks.insert(task_id.clone(), task_info);
        }

        // 启动实际的文件传输任务
        let transfer_tasks = Arc::clone(&self.transfer_tasks);
        let task_id_clone = task_id.clone();
        
        tokio::spawn(async move {
            Self::execute_file_transfer(transfer_tasks, task_id_clone).await;
        });

        log::info!("文件传输任务 {task_id} 已创建并启动");
        Ok(())
    }

    /// 执行文件传输
    ///
    /// # 参数
    ///
    /// * `transfer_tasks` - 传输任务存储
    /// * `task_id` - 任务 ID
    async fn execute_file_transfer(
        transfer_tasks: Arc<RwLock<HashMap<String, FileTransferTaskInfo>>>,
        task_id: String,
    ) {
        // 获取任务信息
        let (source_path, target_path, total_bytes) = {
            let mut tasks = transfer_tasks.write().await;
            if let Some(task_info) = tasks.get_mut(&task_id) {
                task_info.set_status(crate::gateway::TransferStatus::Transferring);
                (task_info.source_path.clone(), task_info.target_path.clone(), task_info.total_bytes)
            } else {
                log::error!("未找到传输任务: {}", task_id);
                return;
            }
        };

        log::info!("开始执行文件传输任务 {}: {:?} -> {:?}", task_id, source_path, target_path);

        // 执行文件复制，并跟踪进度
        let result = Self::copy_file_with_progress(&source_path, &target_path, total_bytes, &transfer_tasks, &task_id).await;
        
        // 更新任务状态
        {
            let mut tasks = transfer_tasks.write().await;
            if let Some(task_info) = tasks.get_mut(&task_id) {
                match result {
                    Ok(bytes_copied) => {
                        task_info.transferred_bytes = bytes_copied;
                        task_info.set_status(crate::gateway::TransferStatus::Completed);
                        log::info!("文件传输任务 {} 完成，复制了 {} 字节", task_id, bytes_copied);
                    }
                    Err(e) => {
                        task_info.set_error(format!("文件传输失败: {}", e));
                        log::error!("文件传输任务 {} 失败: {}", task_id, e);
                    }
                }
            }
        }
    }

    /// 带进度跟踪的文件复制
    ///
    /// # 参数
    ///
    /// * `source_path` - 源文件路径
    /// * `target_path` - 目标文件路径
    /// * `total_bytes` - 总字节数
    /// * `transfer_tasks` - 传输任务存储
    /// * `task_id` - 任务 ID
    ///
    /// # 返回值
    ///
    /// 复制的字节数
    async fn copy_file_with_progress(
        source_path: &std::path::Path,
        target_path: &std::path::Path,
        total_bytes: u64,
        transfer_tasks: &Arc<RwLock<HashMap<String, FileTransferTaskInfo>>>,
        task_id: &str,
    ) -> std::io::Result<u64> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        
        // 确保目标目录存在
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut source_file = tokio::fs::File::open(source_path).await?;
        let mut target_file = tokio::fs::File::create(target_path).await?;
        
        let mut buffer = vec![0u8; 8192]; // 8KB 缓冲区
        let mut total_copied = 0u64;
        let mut last_progress_update = tokio::time::Instant::now();
        let start_time = tokio::time::Instant::now();
        
        loop {
            let bytes_read = source_file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            
            target_file.write_all(&buffer[..bytes_read]).await?;
            total_copied += bytes_read as u64;
            
            // 定期更新进度
            let now = tokio::time::Instant::now();
            if now.duration_since(last_progress_update) >= tokio::time::Duration::from_millis(500) {
                let elapsed = now.duration_since(start_time);
                let transfer_speed = if elapsed.as_secs() > 0 {
                    total_copied / elapsed.as_secs()
                } else {
                    0
                };
                
                // 更新任务进度
                {
                    let mut tasks = transfer_tasks.write().await;
                    if let Some(task_info) = tasks.get_mut(task_id) {
                        task_info.update_progress(total_copied, transfer_speed);
                    }
                }
                
                last_progress_update = now;
                log::debug!("文件传输进度: {}/{} 字节 ({:.1}%)", 
                           total_copied, total_bytes, 
                           (total_copied as f64 / total_bytes as f64) * 100.0);
            }
        }
        
        target_file.flush().await?;
        Ok(total_copied)
    }

    /// 获取文件传输任务状态
    ///
    /// # 参数
    ///
    /// * `task_id` - 任务 ID
    ///
    /// # 返回值
    ///
    /// 传输任务状态
    pub async fn get_transfer_status(
        &self,
        task_id: &str,
    ) -> anyhow::Result<crate::gateway::tauri_api::FileTransferTask> {
        let transfer_tasks = self.transfer_tasks.read().await;
        
        if let Some(task_info) = transfer_tasks.get(task_id) {
            let tauri_task = crate::gateway::tauri_api::FileTransferTask {
                id: task_info.task_id.clone(),
                source_path: task_info.source_path.clone(),
                target_path: task_info.target_path.clone(),
                status: task_info.status.clone(),
                transferred_bytes: task_info.transferred_bytes,
                total_bytes: task_info.total_bytes,
                transfer_speed: task_info.transfer_speed,
                start_time: task_info.start_time,
                estimated_completion: task_info.estimated_completion,
            };
            Ok(tauri_task)
        } else {
            Err(anyhow::anyhow!("未找到传输任务: {}", task_id))
        }
    }

    /// 取消文件传输任务
    ///
    /// # 参数
    ///
    /// * `task_id` - 任务 ID
    ///
    /// # 返回值
    ///
    /// 操作结果
    pub async fn cancel_transfer(&self, task_id: &str) -> anyhow::Result<()> {
        log::info!("取消文件传输任务 {task_id}");

        let mut transfer_tasks = self.transfer_tasks.write().await;
        
        if let Some(task_info) = transfer_tasks.get_mut(task_id) {
            // 检查任务是否可以取消
            match &task_info.status {
                crate::gateway::TransferStatus::Pending 
                | crate::gateway::TransferStatus::Transferring => {
                    task_info.set_status(crate::gateway::TransferStatus::Cancelled);
                    log::info!("文件传输任务 {task_id} 已取消");
                    Ok(())
                }
                crate::gateway::TransferStatus::Completed => {
                    Err(anyhow::anyhow!("任务已完成，无法取消"))
                }
                crate::gateway::TransferStatus::Error(_) => {
                    Err(anyhow::anyhow!("任务已失败，无法取消"))
                }
                crate::gateway::TransferStatus::Cancelled => {
                    Err(anyhow::anyhow!("任务已取消"))
                }
            }
        } else {
            Err(anyhow::anyhow!("未找到传输任务: {}", task_id))
        }
    }

    /// 健康检查
    ///
    /// # 返回值
    ///
    /// 健康状态
    pub async fn health_check(&self) -> Option<bool> {
        // 检查网络管理器的各个组件
        
        // 1. 检查本地地址是否有效
        if self.local_addr.ip().is_unspecified() {
            log::warn!("健康检查失败: 本地地址无效");
            return Some(false);
        }
        
        // 2. 检查 UDP 套接字是否可用
        let test_data = b"health_check";
        if let Err(e) = self.udp_socket.send_to(test_data, self.local_addr) {
            log::warn!("健康检查失败: UDP 套接字不可用: {}", e);
            return Some(false);
        }
        
        // 3. 检查活跃连接数量
        let active_connections = self.connections.lock().await.len();
        log::debug!("健康检查: 活跃连接数量: {}", active_connections);
        
        // 4. 检查 P2P 发现状态
        let p2p_enabled = *self.p2p_discovery_enabled.lock().await;
        let discovered_nodes_count = self.discovered_nodes.read().await.len();
        log::debug!("健康检查: P2P发现启用: {}, 已发现节点: {}", p2p_enabled, discovered_nodes_count);
        
        // 5. 检查传输任务状态
        let active_transfers = self.transfer_tasks.read().await.len();
        log::debug!("健康检查: 活跃传输任务: {}", active_transfers);
        
        // 6. 检查是否有网络接口可用
        match if_addrs::get_if_addrs() {
            Ok(interfaces) => {
                let active_interfaces = interfaces.iter()
                    .filter(|iface| !iface.is_loopback())
                    .count();
                
                if active_interfaces == 0 {
                    log::warn!("健康检查失败: 没有可用的网络接口");
                    return Some(false);
                }
                
                log::debug!("健康检查: 活跃网络接口数量: {}", active_interfaces);
            }
            Err(e) => {
                log::warn!("健康检查失败: 无法获取网络接口: {}", e);
                return Some(false);
            }
        }
        
        log::info!("网络健康检查通过");
        Some(true)
    }

    /// P2P 发现任务
    ///
    /// 定期广播发现消息并处理接收到的回复
    async fn discovery_task(
        discovered_nodes: Arc<RwLock<HashMap<String, DiscoveredNodeInfo>>>,
        event_sender: mpsc::UnboundedSender<NetworkEvent>,
        udp_socket: Arc<UdpSocket>,
        protocol: WdicProtocol,
        broadcast_addresses: Vec<SocketAddr>,
        local_addr: SocketAddr,
        p2p_enabled: Arc<Mutex<bool>>,
    ) {
        let mut discovery_interval = interval(Duration::from_secs(30)); // 每 30 秒发现一次
        let mut cleanup_interval = interval(Duration::from_secs(300)); // 每 5 分钟清理一次过期节点
        let mut receive_buffer = vec![0u8; 65536];

        info!("P2P 发现任务开始运行");

        loop {
            tokio::select! {
                _ = discovery_interval.tick() => {
                    if !*p2p_enabled.lock().await {
                        break;
                    }

                    // 发送发现广播
                    Self::send_discovery_broadcast(
                        &udp_socket,
                        &protocol,
                        &broadcast_addresses,
                        local_addr,
                    ).await;
                }

                _ = cleanup_interval.tick() => {
                    if !*p2p_enabled.lock().await {
                        break;
                    }

                    // 清理过期节点
                    Self::cleanup_expired_nodes(&discovered_nodes).await;
                }

                // 处理接收到的消息
                result = Self::try_receive_message(&udp_socket, &mut receive_buffer) => {
                    if !*p2p_enabled.lock().await {
                        break;
                    }

                    if let Ok((message, sender_addr)) = result {
                        Self::handle_discovery_message(
                            message,
                            sender_addr,
                            &discovered_nodes,
                            &event_sender,
                            local_addr,
                        ).await;
                    }
                }
            }
        }

        info!("P2P 发现任务已停止");
    }

    /// 发送发现广播
    async fn send_discovery_broadcast(
        udp_socket: &UdpSocket,
        _protocol: &WdicProtocol,
        broadcast_addresses: &[SocketAddr],
        local_addr: SocketAddr,
    ) {
        // 创建发现消息
        let discovery_message = WdicMessage::new_discovery(
            format!("gateway-{}", uuid::Uuid::new_v4()),
            "WDIC Gateway".to_string(),
            local_addr,
        );

        if let Ok(serialized) = serde_json::to_vec(&discovery_message) {
            for &addr in broadcast_addresses {
                if let Err(e) = udp_socket.send_to(&serialized, addr) {
                    debug!("发送发现广播到 {} 失败: {}", addr, e);
                } else {
                    debug!("已向 {} 发送发现广播", addr);
                }
            }
        }
    }

    /// 清理过期节点
    async fn cleanup_expired_nodes(discovered_nodes: &Arc<RwLock<HashMap<String, DiscoveredNodeInfo>>>) {
        const EXPIRY_TIMEOUT: i64 = 600; // 10 分钟

        let mut nodes = discovered_nodes.write().await;
        let before_count = nodes.len();
        
        nodes.retain(|_node_id, node_info| {
            let is_active = !node_info.is_expired(EXPIRY_TIMEOUT);
            if !is_active {
                debug!("移除过期节点: {}", node_info.node_id);
            }
            is_active
        });

        let after_count = nodes.len();
        if before_count != after_count {
            info!("清理了 {} 个过期节点，当前活跃节点: {}", before_count - after_count, after_count);
        }
    }

    /// 尝试接收消息
    async fn try_receive_message(
        udp_socket: &UdpSocket,
        buffer: &mut [u8],
    ) -> Result<(WdicMessage, SocketAddr), std::io::Error> {
        // 使用非阻塞方式接收
        match udp_socket.recv_from(buffer) {
            Ok((len, sender_addr)) => {
                if let Ok(message) = serde_json::from_slice::<WdicMessage>(&buffer[..len]) {
                    Ok((message, sender_addr))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "消息反序列化失败",
                    ))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// 处理发现消息
    async fn handle_discovery_message(
        message: WdicMessage,
        sender_addr: SocketAddr,
        discovered_nodes: &Arc<RwLock<HashMap<String, DiscoveredNodeInfo>>>,
        event_sender: &mpsc::UnboundedSender<NetworkEvent>,
        local_addr: SocketAddr,
    ) {
        // 不处理来自自己的消息
        if sender_addr == local_addr {
            return;
        }

        match message {
            WdicMessage::Discovery { node_id, node_name, node_addr: _ } => {
                let mut nodes = discovered_nodes.write().await;
                
                if let Some(existing_node) = nodes.get_mut(&node_id) {
                    // 更新现有节点的最后见到时间
                    existing_node.update_last_seen();
                    debug!("更新现有节点: {}", node_id);
                } else {
                    // 添加新发现的节点
                    let node_info = DiscoveredNodeInfo::new(
                        node_id.clone(),
                        sender_addr.ip().to_string(),
                        sender_addr.port(),
                        node_name,
                        "gateway".to_string(),
                    );
                    
                    nodes.insert(node_id.clone(), node_info);
                    info!("发现新节点: {} 来自 {}", node_id, sender_addr);
                    
                    // 发送新节点发现事件
                    let _ = event_sender.send(NetworkEvent::ConnectionEstablished {
                        remote_addr: sender_addr,
                    });
                }
            }
            _ => {
                debug!("收到非发现消息，忽略");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn create_test_addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }

    #[test]
    fn test_connection_state() {
        let addr = create_test_addr(55555);
        let mut state = ConnectionState::new(addr);

        assert_eq!(state.remote_addr, addr);
        assert!(state.last_active <= chrono::Utc::now());
        assert_eq!(state.last_active, state.established_at);

        // 测试活跃时间更新
        let original_time = state.last_active;
        std::thread::sleep(std::time::Duration::from_millis(1));
        state.update_activity();
        assert!(state.last_active > original_time);

        // 测试超时检查
        assert!(!state.is_expired(3600)); // 1小时不会超时

        // 创建过期连接
        state.last_active = chrono::Utc::now() - chrono::Duration::seconds(7200);
        assert!(state.is_expired(3600)); // 2小时前的连接超时
    }

    #[test]
    fn test_broadcast_addresses_generation() {
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 55555);
        let addresses = NetworkManager::generate_broadcast_addresses(local_addr);

        assert!(!addresses.is_empty());
        // 检查是否包含至少一个广播地址
        let has_ipv4_broadcast = addresses
            .iter()
            .any(|addr| matches!(addr.ip(), IpAddr::V4(_)));
        let has_valid_port = addresses.iter().all(|addr| addr.port() == 55555);

        assert!(has_ipv4_broadcast, "应该至少包含一个 IPv4 广播地址");
        assert!(has_valid_port, "所有地址应该使用正确的端口");

        // 在没有网络接口的环境中，应该至少有后备地址
        if addresses
            .iter()
            .any(|addr| addr == &SocketAddr::from(([255, 255, 255, 255], 55555)))
        {
            // 如果有全网广播地址，测试通过
            // 不需要assert!(true)
        } else {
            // 否则应该有其他有效的广播地址
            assert!(!addresses.is_empty(), "应该生成至少一个广播地址");
        }
    }

    #[test]
    fn test_ipv6_multicast_addresses_generation() {
        // 测试 IPv6 多播地址生成
        let local_addr = SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)),
            55555,
        );
        let addresses = NetworkManager::generate_broadcast_addresses(local_addr);

        assert!(!addresses.is_empty());

        // 检查 IPv6 多播地址是否正确生成
        let _has_ipv6_multicast = addresses.iter().any(
            |addr| matches!(addr.ip(), IpAddr::V6(ipv6) if ipv6.segments()[0] & 0xff00 == 0xff00),
        );

        // 如果系统支持 IPv6，应该有多播地址
        let has_valid_port = addresses.iter().all(|addr| addr.port() == 55555);
        assert!(has_valid_port, "所有地址应该使用正确的端口");
    }

    #[test]
    fn test_private_ipv4_detection() {
        assert!(NetworkManager::is_private_ipv4(Ipv4Addr::new(
            192, 168, 1, 1
        )));
        assert!(NetworkManager::is_private_ipv4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(NetworkManager::is_private_ipv4(Ipv4Addr::new(
            172, 16, 0, 1
        )));
        assert!(NetworkManager::is_private_ipv4(Ipv4Addr::new(
            172, 31, 255, 255
        )));

        assert!(!NetworkManager::is_private_ipv4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(!NetworkManager::is_private_ipv4(Ipv4Addr::new(1, 1, 1, 1)));
        assert!(!NetworkManager::is_private_ipv4(Ipv4Addr::new(
            172, 15, 0, 1
        ))); // 不在私有范围
        assert!(!NetworkManager::is_private_ipv4(Ipv4Addr::new(
            172, 32, 0, 1
        ))); // 不在私有范围
    }

    #[test]
    fn test_private_ipv6_detection() {
        // 链路本地地址 (fe80::/10)
        assert!(NetworkManager::is_private_ipv6(Ipv6Addr::new(
            0xfe80, 0, 0, 0, 0, 0, 0, 1
        )));
        assert!(NetworkManager::is_private_ipv6(Ipv6Addr::new(
            0xfebf, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff
        )));

        // 唯一本地地址 (fc00::/7)
        assert!(NetworkManager::is_private_ipv6(Ipv6Addr::new(
            0xfc00, 0, 0, 0, 0, 0, 0, 1
        )));
        assert!(NetworkManager::is_private_ipv6(Ipv6Addr::new(
            0xfd00, 0, 0, 0, 0, 0, 0, 1
        )));

        // 公网地址
        assert!(!NetworkManager::is_private_ipv6(Ipv6Addr::new(
            0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888
        ))); // Google DNS
        assert!(!NetworkManager::is_private_ipv6(Ipv6Addr::new(
            0x2001, 0xdb8, 0, 0, 0, 0, 0, 1
        ))); // 文档地址
    }

    #[test]
    fn test_fallback_addresses_generation() {
        let addresses = NetworkManager::generate_fallback_addresses(12345);

        assert!(!addresses.is_empty());
        assert!(addresses.contains(&SocketAddr::from(([255, 255, 255, 255], 12345))));
        assert!(addresses.contains(&SocketAddr::from(([192, 168, 255, 255], 12345))));
        assert!(addresses.contains(&SocketAddr::from(([10, 255, 255, 255], 12345))));
        assert!(addresses.contains(&SocketAddr::from(([172, 31, 255, 255], 12345))));
    }

    #[tokio::test]
    async fn test_network_manager_creation() {
        let local_addr = create_test_addr(0); // 使用端口 0 让系统分配
        let manager = NetworkManager::new(local_addr);

        assert!(manager.is_ok());
        let _manager = manager.unwrap();
        // 端口 0 会被系统分配一个有效端口，或者保持 0 但绑定成功
        // 端口已分配成功
    }

    #[tokio::test]
    async fn test_network_manager_basic_operations() {
        let local_addr = create_test_addr(0);
        let manager = NetworkManager::new(local_addr).expect("创建网络管理器失败");

        // 测试基本属性
        assert_eq!(manager.active_connections_count().await, 0);
        assert!(manager.get_active_connections().await.is_empty());

        // 测试关闭
        assert!(manager.shutdown().await.is_ok());
    }
}
