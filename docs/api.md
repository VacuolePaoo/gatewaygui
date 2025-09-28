# 前端API调用文档

本文档详细说明了如何在Vue组件中使用前端API包装函数与Tauri后端进行交互。

## API文件位置

API包装函数位于: `src/lib/gatewayApi.ts`

## 导入方式

在Vue组件中使用以下方式导入需要的API函数:

```typescript
import { functionName } from '@/lib/gatewayApi'
```

## API列表

### 网关核心功能接口

- [startGateway](#startgateway) - 启动网关服务
- [stopGateway](#stopgateway) - 停止网关服务
- [restartGateway](#restartgateway) - 重启网关服务
- [getGatewayStatus](#getgatewaystatus) - 获取网关状态

### 配置管理接口

- [getGatewayConfig](#getgatewayconfig) - 获取网关配置
- [updateGatewayConfig](#updategatewayconfig) - 更新网关配置
- [validateConfig](#validateconfig) - 验证配置有效性
- [resetToDefaultConfig](#resettodefaultconfig) - 重置为默认配置

### 目录和文件操作接口

- [mountDirectory](#mountdirectory) - 挂载目录
- [unmountDirectory](#unmountdirectory) - 卸载目录
- [getMountPoints](#getmountpoints) - 获取挂载点列表
- [listDirectory](#listdirectory) - 列出目录内容
- [createFileTransfer](#createfiletransfer) - 创建文件传输任务
- [getTransferStatus](#gettransferstatus) - 获取文件传输任务状态
- [cancelTransfer](#canceltransfer) - 取消文件传输任务
- [getAllTransfers](#getalltransfers) - 获取所有活跃传输任务
- [cleanupCompletedTransfers](#cleanupcompletedtransfers) - 清理已完成的传输任务
- [createDataTransferRequest](#createdatatransferrequest) - 创建数据传输请求
- [getPendingTransferRequests](#getpendingtransferrequests) - 获取待处理传输请求
- [getTransferRequestDetails](#gettransferrequestdetails) - 获取传输请求详情
- [confirmDataTransfer](#confirmdatatransfer) - 确认数据传输请求

### 网络通信接口

- [getNetworkStatus](#getnetworkstatus) - 获取网络状态
- [startP2pDiscovery](#startp2pdiscovery) - 启动P2P发现
- [stopP2pDiscovery](#stopp2pdiscovery) - 停止P2P发现
- [getDiscoveredNodes](#getdiscoverednodes) - 获取已发现的节点列表
- [connectToNode](#connecttonode) - 连接到指定节点
- [disconnectFromNode](#disconnectfromnode) - 断开与节点的连接
- [getNetworkStats](#getnetworkstats) - 获取网络连接统计信息

### 性能监控接口

- [getPerformanceReport](#getperformancereport) - 获取性能报告
- [getCompressionStats](#getcompressionstats) - 获取压缩统计
- [getCacheStats](#getcachestats) - 获取缓存统计
- [startPerformanceBenchmark](#startperformancebenchmark) - 开始性能基准测试
- [getBenchmarkResult](#getbenchmarkresult) - 获取基准测试结果

### 状态查询接口

- [getSystemInfo](#getsysteminfo) - 获取系统信息
- [getServiceLogs](#getservicelogs) - 获取服务日志
- [healthCheck](#healthcheck) - 健康检查

### 安全管理接口

- [getSecurityConfig](#getsecurityconfig) - 获取安全配置
- [updateSecurityConfig](#updatesecurityconfig) - 更新安全配置
- [generateTlsCertificate](#generatetlscertificate) - 生成TLS证书
- [addAccessRule](#addaccessrule) - 添加访问控制规则
- [removeAccessRule](#removeaccessrule) - 删除访问控制规则
- [getAccessRules](#getaccessrules) - 获取访问控制规则列表
- [validateClientAccess](#validateclientaccess) - 验证客户端访问权限
- [getActiveSessions](#getactivesessions) - 获取活跃会话列表
- [disconnectSession](#disconnectsession) - 强制断开会话

## API详细说明

### 网关核心功能接口

#### startGateway

启动网关服务

**调用方法**:

```typescript
import { startGateway } from '@/lib/gatewayApi'

await startGateway(config)
```

**参数**:

- `config`: [GatewayConfig](#gatewayconfig) - 网关配置对象

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
const config = {
  name: '我的网关',
  port: 55555,
  enable_compression: true,
  max_connections: 100,
  enable_tls: false,
  read_timeout: 30,
  write_timeout: 30,
  idle_timeout: 60
}

try {
  await startGateway(config)
  console.log('网关启动成功')
}
catch (error) {
  console.error('启动网关失败:', error)
}
```

#### stopGateway

停止网关服务

**调用方法**:

```typescript
import { stopGateway } from '@/lib/gatewayApi'

const result = await stopGateway()
```

**参数**: 无

**返回值**:

- `Promise<string>` - 操作结果信息

**示例**:

```typescript
try {
  const result = await stopGateway()
  console.log(result) // 网关服务已停止
}
catch (error) {
  console.error('停止网关失败:', error)
}
```

#### restartGateway

重启网关服务

**调用方法**:

```typescript
import { restartGateway } from '@/lib/gatewayApi'

await restartGateway(config)
```

**参数**:

- `config`: [GatewayConfig](#gatewayconfig) (可选) - 网关配置对象，如果未提供则使用当前配置

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
// 使用当前配置重启
await restartGateway()

// 使用新配置重启
const newConfig = { /* 配置对象 */ }
await restartGateway(newConfig)
```

#### getGatewayStatus

获取网关状态

**调用方法**:

```typescript
import { getGatewayStatus } from '@/lib/gatewayApi'

const status = await getGatewayStatus()
```

**参数**: 无

**返回值**:

- `Promise<GatewayStatus>` - [网关状态对象](#gatewaystatus)

**示例**:

```typescript
const status = await getGatewayStatus()
console.log('运行状态:', status.is_running)
console.log('活跃连接数:', status.active_connections)
```

### 配置管理接口

#### getGatewayConfig

获取网关配置

**调用方法**:

```typescript
import { getGatewayConfig } from '@/lib/gatewayApi'

const config = await getGatewayConfig()
```

**参数**: 无

**返回值**:

- `Promise<GatewayConfig | null>` - [网关配置对象](#gatewayconfig)或null

**示例**:

```typescript
const config = await getGatewayConfig()
if (config) {
  console.log('当前端口:', config.port)
}
```

#### updateGatewayConfig

更新网关配置

**调用方法**:

```typescript
import { updateGatewayConfig } from '@/lib/gatewayApi'

await updateGatewayConfig(config)
```

**参数**:

- `config`: [GatewayConfig](#gatewayconfig) - 网关配置对象

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
const newConfig = { /* 配置对象 */ }
await updateGatewayConfig(newConfig)
```

#### validateConfig

验证配置有效性

**调用方法**:

```typescript
import { validateConfig } from '@/lib/gatewayApi'

const isValid = await validateConfig(config)
```

**参数**:

- `config`: [GatewayConfig](#gatewayconfig) - 网关配置对象

**返回值**:

- `Promise<boolean>` - 配置是否有效

**示例**:

```typescript
const config = { /* 配置对象 */ }
const isValid = await validateConfig(config)
if (isValid) {
  console.log('配置有效')
}
else {
  console.log('配置无效')
}
```

#### resetToDefaultConfig

重置为默认配置

**调用方法**:

```typescript
import { resetToDefaultConfig } from '@/lib/gatewayApi'

const defaultConfig = await resetToDefaultConfig()
```

**参数**: 无

**返回值**:

- `Promise<GatewayConfig>` - [默认配置对象](#gatewayconfig)

**示例**:

```typescript
const defaultConfig = await resetToDefaultConfig()
console.log('默认配置:', defaultConfig)
```

### 目录和文件操作接口

#### mountDirectory

挂载目录

**调用方法**:

```typescript
import { mountDirectory } from '@/lib/gatewayApi'

const mountId = await mountDirectory(localPath, mountName, readOnly)
```

**参数**:

- `localPath`: `string` - 本地路径
- `mountName`: `string` - 挂载名称
- `readOnly`: `boolean` - 是否只读

**返回值**:

- `Promise<string>` - 挂载点ID

**示例**:

```typescript
const mountId = await mountDirectory('/path/to/directory', '我的文档', false)
console.log('挂载ID:', mountId)
```

#### unmountDirectory

卸载目录

**调用方法**:

```typescript
import { unmountDirectory } from '@/lib/gatewayApi'

await unmountDirectory(mountId)
```

**参数**:

- `mountId`: `string` - 挂载点ID

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
await unmountDirectory(mountId)
```

#### getMountPoints

获取挂载点列表

**调用方法**:

```typescript
import { getMountPoints } from '@/lib/gatewayApi'

const mountPoints = await getMountPoints()
```

**参数**: 无

**返回值**:

- `Promise<MountPoint[]>` - [挂载点数组](#mountpoint)

**示例**:

```typescript
const mountPoints = await getMountPoints()
mountPoints.forEach((point) => {
  console.log('挂载点:', point.mount_name)
})
```

#### listDirectory

列出目录内容

**调用方法**:

```typescript
import { listDirectory } from '@/lib/gatewayApi'

const entries = await listDirectory(mountId, path)
```

**参数**:

- `mountId`: `string` - 挂载点ID
- `path`: `string` - 目录路径

**返回值**:

- `Promise<DirectoryEntry[]>` - [目录条目数组](#directoryentry)

**示例**:

```typescript
const entries = await listDirectory(mountId, '/')
entries.forEach((entry) => {
  console.log('文件/目录:', entry.name)
})
```

#### createFileTransfer

创建文件传输任务

**调用方法**:

```typescript
import { createFileTransfer } from '@/lib/gatewayApi'

const taskId = await createFileTransfer(sourcePath, targetPath)
```

**参数**:

- `sourcePath`: `string` - 源文件路径
- `targetPath`: `string` - 目标文件路径

**返回值**:

- `Promise<string>` - 任务ID

**示例**:

```typescript
const taskId = await createFileTransfer('/source/file.txt', '/target/file.txt')
console.log('任务ID:', taskId)
```

#### getTransferStatus

获取文件传输任务状态

**调用方法**:

```typescript
import { getTransferStatus } from '@/lib/gatewayApi'

const task = await getTransferStatus(taskId)
```

**参数**:

- `taskId`: `string` - 任务ID

**返回值**:

- `Promise<FileTransferTask>` - [文件传输任务对象](#filetransfertask)

**示例**:

```typescript
const task = await getTransferStatus(taskId)
console.log('传输状态:', task.status)
```

#### cancelTransfer

取消文件传输任务

**调用方法**:

```typescript
import { cancelTransfer } from '@/lib/gatewayApi'

await cancelTransfer(taskId)
```

**参数**:

- `taskId`: `string` - 任务ID

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
await cancelTransfer(taskId)
```

### 网络通信接口

#### getNetworkStatus

获取网络状态

**调用方法**:

```typescript
import { getNetworkStatus } from '@/lib/gatewayApi'

const networkStatus = await getNetworkStatus()
```

**参数**: 无

**返回值**:

- `Promise<NetworkStatus>` - [网络状态对象](#networkstatus)

**示例**:

```typescript
const networkStatus = await getNetworkStatus()
console.log('本地IP:', networkStatus.local_ip)
```

#### startP2pDiscovery

启动P2P发现

**调用方法**:

```typescript
import { startP2pDiscovery } from '@/lib/gatewayApi'

await startP2pDiscovery()
```

**参数**: 无

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
await startP2pDiscovery()
```

#### stopP2pDiscovery

停止P2P发现

**调用方法**:

```typescript
import { stopP2pDiscovery } from '@/lib/gatewayApi'

await stopP2pDiscovery()
```

**参数**: 无

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
await stopP2pDiscovery()
```

#### getDiscoveredNodes

获取已发现的节点列表

**调用方法**:

```typescript
import { getDiscoveredNodes } from '@/lib/gatewayApi'

const nodes = await getDiscoveredNodes()
```

**参数**: 无

**返回值**:

- `Promise<DiscoveredNode[]>` - [发现节点数组](#discoverednode)

**示例**:

```typescript
const nodes = await getDiscoveredNodes()
nodes.forEach((node) => {
  console.log('节点:', node.name)
})
```

#### connectToNode

连接到指定节点

**调用方法**:

```typescript
import { connectToNode } from '@/lib/gatewayApi'

await connectToNode(nodeId, ipAddress, port)
```

**参数**:

- `nodeId`: `string` - 节点ID
- `ipAddress`: `string` - IP地址
- `port`: `number` - 端口号

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
await connectToNode('node123', '192.168.1.100', 55555)
```

#### disconnectFromNode

断开与节点的连接

**调用方法**:

```typescript
import { disconnectFromNode } from '@/lib/gatewayApi'

await disconnectFromNode(nodeId)
```

**参数**:

- `nodeId`: `string` - 节点ID

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
await disconnectFromNode('node123')
```

### 性能监控接口

#### getPerformanceReport

获取性能报告

**调用方法**:

```typescript
import { getPerformanceReport } from '@/lib/gatewayApi'

const report = await getPerformanceReport()
```

**参数**: 无

**返回值**:

- `Promise<PerformanceReport>` - [性能报告对象](#performancereport)

**示例**:

```typescript
const report = await getPerformanceReport()
console.log('CPU使用率:', report.cpu_usage)
```

#### getCompressionStats

获取压缩统计

**调用方法**:

```typescript
import { getCompressionStats } from '@/lib/gatewayApi'

const stats = await getCompressionStats()
```

**参数**: 无

**返回值**:

- `Promise<CompressionStatsSnapshot>` - [压缩统计对象](#compressionstatssnapshot)

**示例**:

```typescript
const stats = await getCompressionStats()
console.log('压缩率:', stats.compression_ratio)
```

#### getCacheStats

获取缓存统计

**调用方法**:

```typescript
import { getCacheStats } from '@/lib/gatewayApi'

const stats = await getCacheStats()
```

**参数**: 无

**返回值**:

- `Promise<CacheStats>` - [缓存统计对象](#cachestats)

**示例**:

```typescript
const stats = await getCacheStats()
console.log('缓存命中率:', stats.hit_rate)
```

#### startPerformanceBenchmark

开始性能基准测试

**调用方法**:

```typescript
import { startPerformanceBenchmark } from '@/lib/gatewayApi'

const benchmarkId = await startPerformanceBenchmark(testType, durationSeconds)
```

**参数**:

- `testType`: `string` - 测试类型
- `durationSeconds`: `number` - 持续时间（秒）

**返回值**:

- `Promise<string>` - 基准测试ID

**示例**:

```typescript
const benchmarkId = await startPerformanceBenchmark('network', 60)
```

#### getBenchmarkResult

获取基准测试结果

**调用方法**:

```typescript
import { getBenchmarkResult } from '@/lib/gatewayApi'

const result = await getBenchmarkResult(benchmarkId)
```

**参数**:

- `benchmarkId`: `string` - 基准测试ID

**返回值**:

- `Promise<BenchmarkResult>` - [基准测试结果对象](#benchmarkresult)

**示例**:

```typescript
const result = await getBenchmarkResult(benchmarkId)
console.log('测试结果:', result.results)
```

### 状态查询接口

#### getSystemInfo

获取系统信息

**调用方法**:

```typescript
import { getSystemInfo } from '@/lib/gatewayApi'

const systemInfo = await getSystemInfo()
```

**参数**: 无

**返回值**:

- `Promise<SystemInfo>` - [系统信息对象](#systeminfo)

**示例**:

```typescript
const systemInfo = await getSystemInfo()
console.log('操作系统:', systemInfo.os_name)
```

#### getServiceLogs

获取服务日志

**调用方法**:

```typescript
import { getServiceLogs } from '@/lib/gatewayApi'

const logs = await getServiceLogs(lines, level)
```

**参数**:

- `lines`: `number` (可选) - 日志行数
- `level`: `string` (可选) - 日志级别

**返回值**:

- `Promise<LogEntry[]>` - [日志条目数组](#logentry)

**示例**:

```typescript
// 获取所有日志
const allLogs = await getServiceLogs()

// 获取最近10条INFO级别日志
const recentLogs = await getServiceLogs(10, 'INFO')
```

#### healthCheck

健康检查

**调用方法**:

```typescript
import { healthCheck } from '@/lib/gatewayApi'

const health = await healthCheck()
```

**参数**: 无

**返回值**:

- `Promise<HealthStatus>` - [健康状态对象](#healthstatus)

**示例**:

```typescript
const health = await healthCheck()
console.log('总体状态:', health.overall_status)
```

### 安全管理接口

#### getSecurityConfig

获取安全配置

**调用方法**:

```typescript
import { getSecurityConfig } from '@/lib/gatewayApi'

const config = await getSecurityConfig()
```

**参数**: 无

**返回值**:

- `Promise<SecurityConfig>` - [安全配置对象](#securityconfig)

**示例**:

```typescript
const config = await getSecurityConfig()
console.log('TLS启用:', config.tls_enabled)
```

#### updateSecurityConfig

更新安全配置

**调用方法**:

```typescript
import { updateSecurityConfig } from '@/lib/gatewayApi'

await updateSecurityConfig(config)
```

**参数**:

- `config`: [SecurityConfig](#securityconfig) - 安全配置对象

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
const newConfig = { /* 安全配置对象 */ }
await updateSecurityConfig(newConfig)
```

#### generateTlsCertificate

生成TLS证书

**调用方法**:

```typescript
import { generateTlsCertificate } from '@/lib/gatewayApi'

const certificate = await generateTlsCertificate(certInfo)
```

**参数**:

- `certInfo`: [CertificateInfo](#certificateinfo) - 证书信息对象

**返回值**:

- `Promise<GeneratedCertificate>` - [生成的证书对象](#generatedcertificate)

**示例**:

```typescript
const certInfo = {
  common_name: 'example.com',
  organization: 'My Org',
  country: 'CN',
  validity_days: 365,
  subject_alt_names: ['example.com', 'www.example.com']
}

const certificate = await generateTlsCertificate(certInfo)
```

#### addAccessRule

添加访问控制规则

**调用方法**:

```typescript
import { addAccessRule } from '@/lib/gatewayApi'

const ruleId = await addAccessRule(rule)
```

**参数**:

- `rule`: [AccessRule](#accessrule) - 访问控制规则对象

**返回值**:

- `Promise<string>` - 规则ID

**示例**:

```typescript
const rule = {
  id: '',
  name: '测试规则',
  client: '192.168.1.100',
  allowed_paths: ['/public'],
  permissions: ['read'],
  enabled: true
}

const ruleId = await addAccessRule(rule)
```

#### removeAccessRule

删除访问控制规则

**调用方法**:

```typescript
import { removeAccessRule } from '@/lib/gatewayApi'

await removeAccessRule(ruleId)
```

**参数**:

- `ruleId`: `string` - 规则ID

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
await removeAccessRule(ruleId)
```

#### getAccessRules

获取访问控制规则列表

**调用方法**:

```typescript
import { getAccessRules } from '@/lib/gatewayApi'

const rules = await getAccessRules()
```

**参数**: 无

**返回值**:

- `Promise<AccessRule[]>` - [访问控制规则数组](#accessrule)

**示例**:

```typescript
const rules = await getAccessRules()
rules.forEach((rule) => {
  console.log('规则:', rule.name)
})
```

#### validateClientAccess

验证客户端访问权限

**调用方法**:

```typescript
import { validateClientAccess } from '@/lib/gatewayApi'

const isAllowed = await validateClientAccess(clientIp, requestedPath, operation)
```

**参数**:

- `clientIp`: `string` - 客户端IP
- `requestedPath`: `string` - 请求路径
- `operation`: `string` - 操作类型

**返回值**:

- `Promise<boolean>` - 是否允许访问

**示例**:

```typescript
const isAllowed = await validateClientAccess('192.168.1.100', '/public/file.txt', 'read')
if (isAllowed) {
  console.log('访问被允许')
}
else {
  console.log('访问被拒绝')
}
```

#### getActiveSessions

获取活跃会话列表

**调用方法**:

```typescript
import { getActiveSessions } from '@/lib/gatewayApi'

const sessions = await getActiveSessions()
```

**参数**: 无

**返回值**:

- `Promise<ActiveSession[]>` - [活跃会话数组](#activesession)

**示例**:

```typescript
const sessions = await getActiveSessions()
sessions.forEach((session) => {
  console.log('会话:', session.client_ip)
})
```

#### disconnectSession

强制断开会话

**调用方法**:

```typescript
import { disconnectSession } from '@/lib/gatewayApi'

await disconnectSession(sessionId)
```

**参数**:

- `sessionId`: `string` - 会话ID

**返回值**:

- `Promise<void>` - 无返回值

**示例**:

```typescript
await disconnectSession(sessionId)
```

## 数据类型定义

### GatewayStatus

```typescript
interface GatewayStatus {
  is_running: boolean
  start_time: string | null
  config: GatewayConfig | null
  active_connections: number
  total_requests: number
  error_count: number
  uptime_seconds: number
}
```

### GatewayConfig

```typescript
interface GatewayConfig {
  name: string
  port: number
  enable_compression: boolean
  max_connections: number
  enable_tls: boolean
  read_timeout: number
  write_timeout: number
  idle_timeout: number
}
```

### MountPoint

```typescript
interface MountPoint {
  id: string
  local_path: string
  mount_name: string
  read_only: boolean
  mount_time: string
  file_count: number
  total_size: number
}
```

### DirectoryEntry

```typescript
interface DirectoryEntry {
  name: string
  path: string
  is_directory: boolean
  size: number
  modified_time: string
  created_time: string | null
  file_type: string
}
```

### FileTransferTask

```typescript
interface FileTransferTask {
  id: string
  source_path: string
  target_path: string
  status: 'Pending' | 'Transferring' | 'Completed' | 'Cancelled' | 'Error'
  transferred_bytes: number
  total_bytes: number
  transfer_speed: number
  start_time: string
  estimated_completion: string | null
}
```

### NetworkStatus

```typescript
interface NetworkStatus {
  local_ip: string
  listen_port: number
  network_interfaces: NetworkInterface[]
  p2p_discovery_enabled: boolean
  discovered_nodes: number
}
```

### NetworkInterface

```typescript
interface NetworkInterface {
  name: string
  ip_address: string
  is_active: boolean
  interface_type: string
}
```

### DiscoveredNode

```typescript
interface DiscoveredNode {
  node_id: string
  ip_address: string
  port: number
  name: string
  discovered_time: string
  last_seen: string
  is_online: boolean
  node_type: string
}
```

### PerformanceReport

```typescript
interface PerformanceReport {
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
```

### CompressionStatsSnapshot

```typescript
interface CompressionStatsSnapshot {
  total_files: number
  compressed_files: number
  total_original_size: number
  total_compressed_size: number
  compression_ratio: number
  avg_compression_time: number
}
```

### CacheStats

```typescript
interface CacheStats {
  item_count: number
  hit_count: number
  miss_count: number
  hit_rate: number
  memory_usage: number
  max_capacity: number
}
```

### BenchmarkResult

```typescript
interface BenchmarkResult {
  id: string
  test_type: string
  status: string
  start_time: string
  end_time: string | null
  results: Record<string, number>
  error_message: string | null
}
```

### SystemInfo

```typescript
interface SystemInfo {
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
```

### LogEntry

```typescript
interface LogEntry {
  timestamp: string
  level: string
  module: string
  message: string
}
```

### HealthStatus

```typescript
interface HealthStatus {
  overall_status: string
  gateway_status: string
  cache_status: string
  network_status: string
  timestamp: string
}
```

### SecurityConfig

```typescript
interface SecurityConfig {
  tls_enabled: boolean
  cert_path: string | null
  key_path: string | null
  ca_cert_path: string | null
  verify_client_cert: boolean
  allowed_clients: string[]
  access_control_rules: AccessRule[]
}
```

### AccessRule

```typescript
interface AccessRule {
  id: string
  name: string
  client: string
  allowed_paths: string[]
  permissions: string[]
  enabled: boolean
}
```

### CertificateInfo

```typescript
interface CertificateInfo {
  common_name: string
  organization: string
  country: string
  validity_days: number
  subject_alt_names: string[]
}
```

### GeneratedCertificate

```typescript
interface GeneratedCertificate {
  cert_path: string
  key_path: string
  cert_pem: string
  key_pem: string
  generated_time: string
  expiry_time: string
}
```

### ActiveSession

```typescript
interface ActiveSession {
  session_id: string
  client_ip: string
  user_id: string | null
  connect_time: string
  last_activity: string
  bytes_transferred: number
  status: string
}
```

### DataTransferRequest

```typescript
interface DataTransferRequest {
  transfer_id: string
  source_node_id: string
  target_node_id: string | null
  file_path: string
  file_size: number
  request_time: string
  status: 'Pending' | 'Accepted' | 'Rejected' | 'Expired'
}
```

### NetworkStats

```typescript
interface NetworkStats {
  active_connections: number
  discovered_nodes: number
  p2p_discovery_enabled: boolean
  active_transfers: number
  local_address: string
}
```

## 新增API详细说明

### getAllTransfers

获取所有活跃的文件传输任务

```typescript
const transfers = await getAllTransfers()
console.log('活跃传输任务:', transfers)
```

### cleanupCompletedTransfers

清理已完成的文件传输任务

```typescript
const cleanedCount = await cleanupCompletedTransfers()
console.log(`清理了 ${cleanedCount} 个传输任务`)
```

### createDataTransferRequest

创建数据传输请求

```typescript
const transferId = await createDataTransferRequest(
  'source-node-123',
  'target-node-456',
  '/path/to/file.txt',
  1024
)
console.log('传输请求ID:', transferId)
```

### getPendingTransferRequests

获取待处理的传输请求列表

```typescript
const pendingRequests = await getPendingTransferRequests()
console.log('待处理请求:', pendingRequests)
```

### getTransferRequestDetails

获取传输请求详情

```typescript
const details = await getTransferRequestDetails(transferId)
console.log('请求详情:', details)
```

### confirmDataTransfer

确认数据传输请求

```typescript
// 接受传输
await confirmDataTransfer(transferId, true)

// 拒绝传输
await confirmDataTransfer(transferId, false, '文件已存在')
```

### getNetworkStats

获取网络连接统计信息

```typescript
const stats = await getNetworkStats()
console.log('网络统计:', stats)
```

## 错误处理

所有API调用都可能抛出错误，建议使用try/catch进行处理:

```typescript
import { startGateway } from '@/lib/gatewayApi'

try {
  await startGateway(config)
  // 处理成功情况
}
catch (error) {
  console.error('API调用失败:', error)
  // 处理错误情况
}
```

## 类型定义

所有API函数都有完整的TypeScript类型定义，提供良好的类型检查和自动补全支持。
