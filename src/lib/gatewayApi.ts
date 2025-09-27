import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

/**
 * 网关核心功能接口
 */

// 事件类型定义
export interface GatewayEvent {
  type: string
  data: any
  timestamp: string
}

export interface NodeDiscoveryEvent {
  node_id: string
  ip_address: string
  port: number
  name: string
  node_type: string
}

export interface DataTransferEvent {
  transfer_id: string
  status: 'requested' | 'accepted' | 'rejected' | 'completed' | 'failed'
  metadata?: any
  error?: string
}

export interface ExceptionEvent {
  error_type: string
  message: string
  details?: any
}

// 回调函数类型
export type EventCallback<T = any> = (data: T) => void | Promise<void>

// 全局事件监听器存储
const eventListeners: Map<string, EventCallback[]> = new Map()

/**
 * 注册事件监听器
 * @param eventType 事件类型
 * @param callback 回调函数
 */
export function addEventListener<T = any>(eventType: string, callback: EventCallback<T>): void {
  if (!eventListeners.has(eventType)) {
    eventListeners.set(eventType, [])
  }
  eventListeners.get(eventType)!.push(callback)
}

/**
 * 移除事件监听器
 * @param eventType 事件类型
 * @param callback 回调函数
 */
export function removeEventListener<T = any>(eventType: string, callback: EventCallback<T>): void {
  const listeners = eventListeners.get(eventType)
  if (listeners) {
    const index = listeners.indexOf(callback)
    if (index > -1) {
      listeners.splice(index, 1)
    }
  }
}

/**
 * 初始化事件监听
 */
export async function initializeEventListeners(): Promise<void> {
  // 监听新节点发现事件
  await listen<NodeDiscoveryEvent>('node-discovered', (event) => {
    const listeners = eventListeners.get('node-discovered')
    if (listeners) {
      listeners.forEach(callback => callback(event.payload))
    }
  })

  // 监听数据传输事件
  await listen<DataTransferEvent>('data-transfer', (event) => {
    const listeners = eventListeners.get('data-transfer')
    if (listeners) {
      listeners.forEach(callback => callback(event.payload))
    }
  })

  // 监听异常事件
  await listen<ExceptionEvent>('gateway-exception', (event) => {
    const listeners = eventListeners.get('gateway-exception')
    if (listeners) {
      listeners.forEach(callback => callback(event.payload))
    }
  })

  // 监听缓存统计更新事件
  await listen('cache-stats-updated', (event) => {
    const listeners = eventListeners.get('cache-stats-updated')
    if (listeners) {
      listeners.forEach(callback => callback(event.payload))
    }
  })
}

/**
 * 确认数据传输请求
 * @param transferId 传输ID
 * @param accept 是否接受
 * @param reason 拒绝原因（如果拒绝）
 */
export async function confirmDataTransfer(
  transferId: string,
  accept: boolean,
  reason?: string,
): Promise<void> {
  return await invoke('confirm_data_transfer', { 
    transfer_id: transferId, 
    accept, 
    reason 
  })
}

// 网关状态信息
export interface GatewayStatus {
  is_running: boolean
  start_time: string | null
  config: GatewayConfig | null
  active_connections: number
  total_requests: number
  error_count: number
  uptime_seconds: number
}

// 网关配置信息
export interface GatewayConfig {
  name: string
  port: number
  enable_compression: boolean
  max_connections: number
  enable_tls: boolean
  read_timeout: number
  write_timeout: number
  idle_timeout: number
}

/**
 * 启动网关服务
 * @param config 网关配置
 * @returns 操作结果
 */
export async function startGateway(config: GatewayConfig): Promise<void> {
  return await invoke('start_gateway', { config })
}

/**
 * 停止网关服务
 * @returns 操作结果信息
 */
export async function stopGateway(): Promise<string> {
  return await invoke('stop_gateway')
}

/**
 * 重启网关服务
 * @param config 网关配置（可选）
 * @returns 操作结果
 */
export async function restartGateway(config?: GatewayConfig): Promise<void> {
  return await invoke('restart_gateway', { config })
}

/**
 * 获取网关状态
 * @returns 网关状态信息
 */
export async function getGatewayStatus(): Promise<GatewayStatus> {
  return await invoke('get_gateway_status')
}

/**
 * 配置管理接口
 */

/**
 * 获取网关配置
 * @returns 网关配置信息
 */
export async function getGatewayConfig(): Promise<GatewayConfig | null> {
  return await invoke('get_gateway_config')
}

/**
 * 更新网关配置
 * @param config 网关配置
 * @returns 操作结果
 */
export async function updateGatewayConfig(config: GatewayConfig): Promise<void> {
  return await invoke('update_gateway_config', { config })
}

/**
 * 验证配置有效性
 * @param config 网关配置
 * @returns 验证结果
 */
export async function validateConfig(config: GatewayConfig): Promise<boolean> {
  return await invoke('validate_config', { config })
}

/**
 * 重置为默认配置
 * @returns 默认配置
 */
export async function resetToDefaultConfig(): Promise<GatewayConfig> {
  return await invoke('reset_to_default_config')
}

/**
 * 目录和文件操作接口
 */

// 挂载点信息
export interface MountPoint {
  id: string
  local_path: string
  mount_name: string
  read_only: boolean
  mount_time: string
  file_count: number
  total_size: number
}

// 目录条目信息
export interface DirectoryEntry {
  name: string
  path: string
  is_directory: boolean
  size: number
  modified_time: string
  created_time: string | null
  file_type: string
}

// 文件传输任务信息
export interface FileTransferTask {
  id: string
  source_path: string
  target_path: string
  status: TransferStatus
  transferred_bytes: number
  total_bytes: number
  transfer_speed: number
  start_time: string
  estimated_completion: string | null
}

// 传输状态枚举
export type TransferStatus = 'Pending' | 'Transferring' | 'Completed' | 'Cancelled' | 'Error'

/**
 * 挂载目录
 * @param localPath 本地路径
 * @param mountName 挂载名称
 * @param readOnly 是否只读
 * @returns 挂载点ID
 */
export async function mountDirectory(
  localPath: string,
  mountName: string,
  readOnly: boolean,
): Promise<string> {
  return await invoke('mount_directory', { 
    local_path: localPath, 
    mount_name: mountName, 
    read_only: readOnly 
  })
}

/**
 * 卸载目录
 * @param mountId 挂载点ID
 * @returns 操作结果
 */
export async function unmountDirectory(mountId: string): Promise<void> {
  return await invoke('unmount_directory', { mount_id: mountId })
}

/**
 * 获取挂载点列表
 * @returns 挂载点列表
 */
export async function getMountPoints(): Promise<MountPoint[]> {
  return await invoke('get_mount_points')
}

/**
 * 列出目录内容
 * @param mountId 挂载点ID
 * @param path 路径
 * @returns 目录条目列表
 */
export async function listDirectory(mountId: string, path: string): Promise<DirectoryEntry[]> {
  return await invoke('list_directory', { mount_id: mountId, path })
}

/**
 * 创建搜索令牌
 * @param mountId 挂载点ID
 * @param patterns 搜索模式列表
 * @param permissions 权限列表
 * @param ttlSeconds 生存时间（秒）
 * @returns 搜索令牌ID
 */
export async function createSearchToken(
  mountId: string,
  patterns: string[],
  permissions: string[],
  ttlSeconds: number,
): Promise<string> {
  return await invoke('create_search_token', { mountId, patterns, permissions, ttlSeconds })
}

/**
 * 验证搜索令牌
 * @param tokenId 令牌ID
 * @param path 要访问的路径
 * @returns 是否授权
 */
export async function validateSearchToken(tokenId: string, path: string): Promise<boolean> {
  return await invoke('validate_search_token', { tokenId, path })
}

/**
 * 文件授权
 * @param filePath 文件路径
 * @param authType 授权类型
 * @param permissions 权限列表
 * @returns 授权ID
 */
export async function authorizeFile(
  filePath: string,
  authType: string,
  permissions: string[],
): Promise<string> {
  return await invoke('authorize_file', { filePath, authType, permissions })
}

/**
 * 通过搜索令牌获取元数据
 * @param tokenId 搜索令牌ID
 * @returns 文件元数据列表
 */
export async function getMetadataByToken(tokenId: string): Promise<Record<string, string>[]> {
  return await invoke('get_metadata_by_token', { tokenId })
}

/**
 * 创建文件传输任务
 * @param sourcePath 源路径
 * @param targetPath 目标路径
 * @returns 任务ID
 */
export async function createFileTransfer(
  sourcePath: string,
  targetPath: string,
): Promise<string> {
  return await invoke('create_file_transfer', { sourcePath, targetPath })
}

/**
 * 获取文件传输任务状态
 * @param taskId 任务ID
 * @returns 文件传输任务信息
 */
export async function getTransferStatus(taskId: string): Promise<FileTransferTask> {
  return await invoke('get_transfer_status', { task_id: taskId })
}

/**
 * 取消文件传输任务
 * @param taskId 任务ID
 * @returns 操作结果
 */
export async function cancelTransfer(taskId: string): Promise<void> {
  return await invoke('cancel_transfer', { task_id: taskId })
}

/**
 * 网络通信接口
 */

// 网络状态信息
export interface NetworkStatus {
  local_ip: string
  listen_port: number
  network_interfaces: NetworkInterface[]
  p2p_discovery_enabled: boolean
  discovered_nodes: number
}

// 网络接口信息
export interface NetworkInterface {
  name: string
  ip_address: string
  is_active: boolean
  interface_type: string
}

// 发现的节点信息
export interface DiscoveredNode {
  node_id: string
  ip_address: string
  port: number
  name: string
  discovered_time: string
  last_seen: string
  is_online: boolean
  node_type: string
}

/**
 * 获取网络状态
 * @returns 网络状态信息
 */
export async function getNetworkStatus(): Promise<NetworkStatus> {
  return await invoke('get_network_status')
}

/**
 * 启动P2P发现
 * @returns 操作结果
 */
export async function startP2pDiscovery(): Promise<void> {
  return await invoke('start_p2p_discovery')
}

/**
 * 停止P2P发现
 * @returns 操作结果
 */
export async function stopP2pDiscovery(): Promise<void> {
  return await invoke('stop_p2p_discovery')
}

/**
 * 获取已发现的节点列表
 * @returns 节点列表
 */
export async function getDiscoveredNodes(): Promise<DiscoveredNode[]> {
  return await invoke('get_discovered_nodes')
}

/**
 * 连接到指定节点
 * @param nodeId 节点ID
 * @param ipAddress IP地址
 * @param port 端口
 * @returns 操作结果
 */
export async function connectToNode(
  nodeId: string,
  ipAddress: string,
  port: number,
): Promise<void> {
  return await invoke('connect_to_node', { node_id: nodeId, ip_address: ipAddress, port })
}

/**
 * 断开与节点的连接
 * @param nodeId 节点ID
 * @returns 操作结果
 */
export async function disconnectFromNode(nodeId: string): Promise<void> {
  return await invoke('disconnect_from_node', { node_id: nodeId })
}

/**
 * 性能监控接口
 */

// 性能报告信息
export interface PerformanceReport {
  uptime_seconds: number
  total_requests: number
  current_connections: number
  error_count: number
  avg_response_time: number
  total_transferred_bytes: number
  current_transfer_rate: number
  cpu_usage: number
  memory_usage: number
}

// 压缩统计信息
export interface CompressionStatsSnapshot {
  total_files: number
  compressed_files: number
  total_original_size: number
  total_compressed_size: number
  compression_ratio: number
  avg_compression_time: number
}

// 缓存统计信息
export interface CacheStats {
  item_count: number
  hit_count: number
  miss_count: number
  hit_rate: number
  memory_usage: number
  max_capacity: number
}

// 基准测试结果
export interface BenchmarkResult {
  id: string
  test_type: string
  status: string
  start_time: string
  end_time: string | null
  results: Record<string, number>
  error_message: string | null
}

/**
 * 获取性能报告
 * @returns 性能报告
 */
export async function getPerformanceReport(): Promise<PerformanceReport> {
  return await invoke('get_performance_report')
}

/**
 * 获取压缩统计
 * @returns 压缩统计信息
 */
export async function getCompressionStats(): Promise<CompressionStatsSnapshot> {
  return await invoke('get_compression_stats')
}

/**
 * 获取缓存统计
 * @returns 缓存统计信息
 */
export async function getCacheStats(): Promise<CacheStats> {
  return await invoke('get_cache_stats')
}

/**
 * 开始性能基准测试
 * @param testType 测试类型
 * @param durationSeconds 持续时间（秒）
 * @returns 基准测试ID
 */
export async function startPerformanceBenchmark(
  testType: string,
  durationSeconds: number,
): Promise<string> {
  return await invoke('start_performance_benchmark', { testType, durationSeconds })
}

/**
 * 获取基准测试结果
 * @param benchmarkId 基准测试ID
 * @returns 基准测试结果
 */
export async function getBenchmarkResult(benchmarkId: string): Promise<BenchmarkResult> {
  return await invoke('get_benchmark_result', { benchmark_id: benchmarkId })
}

/**
 * 状态查询接口
 */

// 系统信息
export interface SystemInfo {
  os_name: string
  os_version: string
  kernel_version: string
  host_name: string
  cpu_count: number
  cpu_usage: number
  total_memory: number
  used_memory: number
  available_memory: number
  uptime: number
}

// 日志条目
export interface LogEntry {
  timestamp: string
  level: string
  module: string
  message: string
}

// 健康状态
export interface HealthStatus {
  overall_status: string
  gateway_status: string
  cache_status: string
  network_status: string
  timestamp: string
}

/**
 * 获取系统信息
 * @returns 系统信息
 */
export async function getSystemInfo(): Promise<SystemInfo> {
  return await invoke('get_system_info')
}

/**
 * 获取服务日志
 * @param lines 行数（可选）
 * @param level 日志级别（可选）
 * @returns 日志条目列表
 */
export async function getServiceLogs(
  lines?: number,
  level?: string,
): Promise<LogEntry[]> {
  return await invoke('get_service_logs', { lines, level })
}

/**
 * 健康检查
 * @returns 健康状态
 */
export async function healthCheck(): Promise<HealthStatus> {
  return await invoke('health_check')
}

/**
 * 安全管理接口
 */

// 安全配置
export interface SecurityConfig {
  tls_enabled: boolean
  cert_path: string | null
  key_path: string | null
  ca_cert_path: string | null
  verify_client_cert: boolean
  allowed_clients: string[]
  access_control_rules: AccessRule[]
}

// 访问控制规则
export interface AccessRule {
  id: string
  name: string
  client: string
  allowed_paths: string[]
  permissions: string[]
  enabled: boolean
}

// 证书信息
export interface CertificateInfo {
  common_name: string
  organization: string
  country: string
  validity_days: number
  subject_alt_names: string[]
}

// 生成的证书
export interface GeneratedCertificate {
  cert_path: string
  key_path: string
  cert_pem: string
  key_pem: string
  generated_time: string
  expiry_time: string
}

// 活跃会话信息
export interface ActiveSession {
  session_id: string
  client_ip: string
  user_id: string | null
  connect_time: string
  last_activity: string
  bytes_transferred: number
  status: string
}

/**
 * 获取安全配置
 * @returns 安全配置
 */
export async function getSecurityConfig(): Promise<SecurityConfig> {
  return await invoke('get_security_config')
}

/**
 * 更新安全配置
 * @param config 安全配置
 * @returns 操作结果
 */
export async function updateSecurityConfig(config: SecurityConfig): Promise<void> {
  return await invoke('update_security_config', { config })
}

/**
 * 生成TLS证书
 * @param certInfo 证书信息
 * @returns 生成的证书
 */
export async function generateTlsCertificate(
  certInfo: CertificateInfo,
): Promise<GeneratedCertificate> {
  return await invoke('generate_tls_certificate', { cert_info: certInfo })
}

/**
 * 添加访问控制规则
 * @param rule 访问控制规则
 * @returns 规则ID
 */
export async function addAccessRule(rule: AccessRule): Promise<string> {
  return await invoke('add_access_rule', { rule })
}

/**
 * 删除访问控制规则
 * @param ruleId 规则ID
 * @returns 操作结果
 */
export async function removeAccessRule(ruleId: string): Promise<void> {
  return await invoke('remove_access_rule', { rule_id: ruleId })
}

/**
 * 获取访问控制规则列表
 * @returns 访问控制规则列表
 */
export async function getAccessRules(): Promise<AccessRule[]> {
  return await invoke('get_access_rules')
}

/**
 * 验证客户端访问权限
 * @param clientIp 客户端IP
 * @param requestedPath 请求路径
 * @param operation 操作类型
 * @returns 是否允许访问
 */
export async function validateClientAccess(
  clientIp: string,
  requestedPath: string,
  operation: string,
): Promise<boolean> {
  return await invoke('validate_client_access', { 
    client_ip: clientIp, 
    requested_path: requestedPath, 
    operation 
  })
}

/**
 * 获取活跃会话列表
 * @returns 活跃会话列表
 */
export async function getActiveSessions(): Promise<ActiveSession[]> {
  return await invoke('get_active_sessions')
}

/**
 * 强制断开会话
 * @param sessionId 会话ID
 * @returns 操作结果
 */
export async function disconnectSession(sessionId: string): Promise<void> {
  return await invoke('disconnect_session', { session_id: sessionId })
}

/**
 * 创建数据传输请求
 * @param sourceNodeId 源节点ID
 * @param targetNodeId 目标节点ID（可选）
 * @param filePath 文件路径
 * @param fileSize 文件大小
 * @returns 传输请求ID
 */
export async function createDataTransferRequest(
  sourceNodeId: string,
  targetNodeId: string | null,
  filePath: string,
  fileSize: number,
): Promise<string> {
  return await invoke('create_data_transfer_request', {
    source_node_id: sourceNodeId,
    target_node_id: targetNodeId,
    file_path: filePath,
    file_size: fileSize,
  })
}

/**
 * 获取待处理的传输请求列表
 * @returns 待处理传输请求列表
 */
export async function getPendingTransferRequests(): Promise<DataTransferRequest[]> {
  return await invoke('get_pending_transfer_requests')
}

/**
 * 获取传输请求详情
 * @param transferId 传输请求ID
 * @returns 传输请求详情
 */
export async function getTransferRequestDetails(transferId: string): Promise<DataTransferRequest> {
  return await invoke('get_transfer_request_details', { transfer_id: transferId })
}

// 数据传输请求信息
export interface DataTransferRequest {
  transfer_id: string
  source_node_id: string
  target_node_id: string | null
  file_path: string
  file_size: number
  request_time: string
  status: DataTransferRequestStatus
}

// 数据传输请求状态
export type DataTransferRequestStatus = 'Pending' | 'Accepted' | 'Rejected' | 'Expired'

// 网络统计信息
export interface NetworkStats {
  active_connections: number
  discovered_nodes: number
  p2p_discovery_enabled: boolean
  active_transfers: number
  local_address: string
}

/**
 * 获取所有活跃的文件传输任务
 * @returns 所有传输任务列表
 */
export async function getAllTransfers(): Promise<FileTransferTask[]> {
  return await invoke('get_all_transfers')
}

/**
 * 清理已完成的文件传输任务
 * @returns 清理的任务数量
 */
export async function cleanupCompletedTransfers(): Promise<number> {
  return await invoke('cleanup_completed_transfers')
}

/**
 * 获取网络连接统计信息
 * @returns 网络统计信息
 */
export async function getNetworkStats(): Promise<NetworkStats> {
  return await invoke('get_network_stats')
}
