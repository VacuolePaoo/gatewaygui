//! 目录挂载模块
//!
//! 处理目录挂载、文件授权、搜索令牌系统和文件路径安全处理。
//! 此模块独立于注册表，专门负责文件系统访问控制和挂载点管理。

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

use crate::gateway::tauri_api::{DirectoryEntry, MountPoint};

/// 搜索令牌信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchToken {
    /// 令牌 ID
    pub token_id: String,
    /// 关联的挂载点 ID
    pub mount_id: String,
    /// 允许的路径模式
    pub allowed_patterns: Vec<String>,
    /// 令牌权限
    pub permissions: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 是否激活
    pub is_active: bool,
}

impl SearchToken {
    /// 创建新的搜索令牌
    pub fn new(
        mount_id: String,
        allowed_patterns: Vec<String>,
        permissions: Vec<String>,
        ttl_seconds: u64,
    ) -> Self {
        let now = Utc::now();
        Self {
            token_id: Uuid::new_v4().to_string(),
            mount_id,
            allowed_patterns,
            permissions,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_seconds as i64),
            is_active: true,
        }
    }

    /// 检查令牌是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 检查路径是否被令牌授权
    pub fn is_path_authorized(&self, path: &str) -> bool {
        if !self.is_active || self.is_expired() {
            return false;
        }

        // 检查路径是否匹配任何允许的模式
        self.allowed_patterns.iter().any(|pattern| {
            // 简单的模式匹配，支持通配符
            if pattern.ends_with("*") {
                let prefix = &pattern[..pattern.len() - 1];
                path.starts_with(prefix)
            } else {
                path == pattern
            }
        })
    }
}

/// 文件授权信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAuthorization {
    /// 授权 ID
    pub auth_id: String,
    /// 文件路径
    pub file_path: PathBuf,
    /// 授权类型
    pub auth_type: String,
    /// 权限列表
    pub permissions: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 是否激活
    pub is_active: bool,
}

impl FileAuthorization {
    /// 创建新的文件授权
    pub fn new(file_path: PathBuf, auth_type: String, permissions: Vec<String>) -> Self {
        Self {
            auth_id: Uuid::new_v4().to_string(),
            file_path,
            auth_type,
            permissions,
            created_at: Utc::now(),
            is_active: true,
        }
    }
}

/// 目录挂载管理器
#[derive(Debug)]
pub struct MountManager {
    /// 挂载点存储 (mount_id -> MountPoint)
    mount_points: Arc<DashMap<String, MountPoint>>,
    /// 搜索令牌存储 (token_id -> SearchToken)
    search_tokens: Arc<DashMap<String, SearchToken>>,
    /// 文件授权存储 (auth_id -> FileAuthorization)
    file_authorizations: Arc<DashMap<String, FileAuthorization>>,
    /// 路径到授权 ID 的映射
    path_to_auth: Arc<DashMap<String, String>>,
}

impl Default for MountManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MountManager {
    /// 创建新的挂载管理器
    pub fn new() -> Self {
        Self {
            mount_points: Arc::new(DashMap::new()),
            search_tokens: Arc::new(DashMap::new()),
            file_authorizations: Arc::new(DashMap::new()),
            path_to_auth: Arc::new(DashMap::new()),
        }
    }

    /// 挂载目录
    ///
    /// # 参数
    ///
    /// * `mount_point` - 挂载点信息
    ///
    /// # 返回值
    ///
    /// 操作结果
    pub async fn mount_directory(&self, mut mount_point: MountPoint) -> Result<String> {
        // 验证路径存在且是目录
        if !mount_point.local_path.exists() {
            return Err(anyhow!("路径不存在: {:?}", mount_point.local_path));
        }

        if !mount_point.local_path.is_dir() {
            return Err(anyhow!("不是目录: {:?}", mount_point.local_path));
        }

        // 安全路径处理：规范化路径
        let canonical_path = fs::canonicalize(&mount_point.local_path)
            .context("无法规范化路径")?;
        
        mount_point.local_path = canonical_path;

        // 计算目录统计信息
        let (file_count, total_size) = self.calculate_directory_stats(&mount_point.local_path)?;
        mount_point.file_count = file_count;
        mount_point.total_size = total_size;

        let mount_id = mount_point.id.clone();
        self.mount_points.insert(mount_id.clone(), mount_point.clone());

        info!("成功挂载目录: {} -> {:?}", mount_id, mount_point.local_path);
        Ok(mount_id)
    }

    /// 卸载目录
    ///
    /// # 参数
    ///
    /// * `mount_id` - 挂载点 ID
    ///
    /// # 返回值
    ///
    /// 操作结果
    pub async fn unmount_directory(&self, mount_id: &str) -> Result<()> {
        let mount_point = self.mount_points.remove(mount_id)
            .ok_or_else(|| anyhow!("挂载点不存在: {mount_id}"))?;

        // 清理相关的搜索令牌
        let mut tokens_to_remove = Vec::new();
        for token_entry in self.search_tokens.iter() {
            if token_entry.value().mount_id == mount_id {
                tokens_to_remove.push(token_entry.key().clone());
            }
        }

        for token_id in tokens_to_remove {
            self.search_tokens.remove(&token_id);
        }

        info!("成功卸载目录: {} -> {:?}", mount_id, mount_point.1.local_path);
        Ok(())
    }

    /// 获取挂载点列表
    pub async fn get_mount_points(&self) -> Result<Vec<MountPoint>> {
        let mut mount_points: Vec<MountPoint> = self
            .mount_points
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        // 按挂载时间排序
        mount_points.sort_by(|a, b| b.mount_time.cmp(&a.mount_time));

        Ok(mount_points)
    }

    /// 列出目录内容
    ///
    /// # 参数
    ///
    /// * `mount_id` - 挂载点 ID
    /// * `relative_path` - 相对路径
    ///
    /// # 返回值
    ///
    /// 目录条目列表
    pub async fn list_directory(
        &self,
        mount_id: &str,
        relative_path: &str,
    ) -> Result<Vec<DirectoryEntry>> {
        let mount_point = self.mount_points.get(mount_id)
            .ok_or_else(|| anyhow!("挂载点不存在: {mount_id}"))?;

        let mut target_path = mount_point.local_path.clone();
        if !relative_path.is_empty() && relative_path != "/" {
            target_path.push(relative_path.trim_start_matches('/'));
        }

        // 安全检查：确保路径在挂载点范围内
        self.validate_path_security(&mount_point.local_path, &target_path)?;

        if !target_path.exists() {
            return Err(anyhow!("路径不存在: {:?}", target_path));
        }

        if !target_path.is_dir() {
            return Err(anyhow!("不是目录: {:?}", target_path));
        }

        let mut entries = Vec::new();
        let read_dir = fs::read_dir(&target_path)?;

        for entry in read_dir {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let full_path = entry.path();

            let modified_time = metadata.modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            let modified_time = DateTime::from_timestamp(modified_time as i64, 0)
                .unwrap_or_else(Utc::now);

            let created_time = metadata.created().ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .and_then(|duration| DateTime::from_timestamp(duration.as_secs() as i64, 0));

            entries.push(DirectoryEntry {
                name: file_name.clone(),
                path: full_path.to_string_lossy().to_string(),
                is_directory: metadata.is_dir(),
                size: if metadata.is_file() { metadata.len() } else { 0 },
                modified_time,
                created_time,
                file_type: if metadata.is_dir() {
                    "directory".to_string()
                } else if metadata.is_file() {
                    // 文件类型推断
                    if let Some(extension) = full_path.extension() {
                        extension.to_string_lossy().to_string()
                    } else {
                        "file".to_string()
                    }
                } else {
                    "unknown".to_string()
                },
            });
        }

        // 按名称排序，目录在前
        entries.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        Ok(entries)
    }

    /// 创建搜索令牌
    ///
    /// # 参数
    ///
    /// * `mount_id` - 挂载点 ID
    /// * `patterns` - 搜索模式
    /// * `permissions` - 权限列表
    /// * `ttl_seconds` - 生存时间（秒）
    ///
    /// # 返回值
    ///
    /// 搜索令牌 ID
    pub async fn create_search_token(
        &self,
        mount_id: String,
        patterns: Vec<String>,
        permissions: Vec<String>,
        ttl_seconds: u64,
    ) -> Result<String> {
        // 验证挂载点存在
        if !self.mount_points.contains_key(&mount_id) {
            return Err(anyhow!("挂载点不存在: {mount_id}"));
        }

        let token = SearchToken::new(mount_id, patterns, permissions, ttl_seconds);
        let token_id = token.token_id.clone();
        
        self.search_tokens.insert(token_id.clone(), token);
        
        info!("创建搜索令牌: {token_id}");
        Ok(token_id)
    }

    /// 验证搜索令牌
    ///
    /// # 参数
    ///
    /// * `token_id` - 令牌 ID
    /// * `path` - 要访问的路径
    ///
    /// # 返回值
    ///
    /// 是否授权
    pub async fn validate_search_token(&self, token_id: &str, path: &str) -> Result<bool> {
        let token = self.search_tokens.get(token_id)
            .ok_or_else(|| anyhow!("搜索令牌不存在: {token_id}"))?;

        Ok(token.is_path_authorized(path))
    }

    /// 文件授权函数
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    /// * `auth_type` - 授权类型
    /// * `permissions` - 权限列表
    ///
    /// # 返回值
    ///
    /// 授权 ID
    pub async fn authorize_file(
        &self,
        file_path: PathBuf,
        auth_type: String,
        permissions: Vec<String>,
    ) -> Result<String> {
        // 验证文件存在
        if !file_path.exists() {
            return Err(anyhow!("文件不存在: {:?}", file_path));
        }

        // 安全路径处理
        let canonical_path = fs::canonicalize(&file_path)
            .context("无法规范化文件路径")?;

        // 检查文件是否在任何挂载点范围内
        let mut is_in_mount = false;
        for mount_entry in self.mount_points.iter() {
            if canonical_path.starts_with(&mount_entry.value().local_path) {
                is_in_mount = true;
                break;
            }
        }

        if !is_in_mount {
            return Err(anyhow!("文件不在任何挂载点范围内: {:?}", canonical_path));
        }

        let authorization = FileAuthorization::new(canonical_path.clone(), auth_type, permissions);
        let auth_id = authorization.auth_id.clone();
        
        self.file_authorizations.insert(auth_id.clone(), authorization);
        self.path_to_auth.insert(canonical_path.to_string_lossy().to_string(), auth_id.clone());

        info!("文件授权成功: {} -> {:?}", auth_id, canonical_path);
        Ok(auth_id)
    }

    /// 通过搜索令牌获取元数据
    ///
    /// # 参数
    ///
    /// * `token_id` - 搜索令牌 ID
    ///
    /// # 返回值
    ///
    /// 文件元数据列表
    pub async fn get_metadata_by_token(&self, token_id: &str) -> Result<Vec<HashMap<String, String>>> {
        let token = self.search_tokens.get(token_id)
            .ok_or_else(|| anyhow!("搜索令牌不存在: {token_id}"))?;

        if token.is_expired() {
            return Err(anyhow!("搜索令牌已过期: {token_id}"));
        }

        let mount_point = self.mount_points.get(&token.mount_id)
            .ok_or_else(|| anyhow!("挂载点不存在: {}", token.mount_id))?;

        let mut metadata_list = Vec::new();
        
        // 搜索符合模式的文件
        for pattern in &token.allowed_patterns {
            let files = self.search_files_by_pattern(&mount_point.local_path, pattern)?;
            for file_path in files {
                if let Ok(metadata) = fs::metadata(&file_path) {
                    let mut file_metadata = HashMap::new();
                    file_metadata.insert("path".to_string(), file_path.to_string_lossy().to_string());
                    file_metadata.insert("size".to_string(), metadata.len().to_string());
                    file_metadata.insert("is_file".to_string(), metadata.is_file().to_string());
                    file_metadata.insert("is_dir".to_string(), metadata.is_dir().to_string());
                    
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                            file_metadata.insert("modified".to_string(), duration.as_secs().to_string());
                        }
                    }
                    
                    metadata_list.push(file_metadata);
                }
            }
        }

        Ok(metadata_list)
    }

    /// 清理过期令牌
    pub async fn cleanup_expired_tokens(&self) -> Result<usize> {
        let mut expired_tokens = Vec::new();
        
        for token_entry in self.search_tokens.iter() {
            if token_entry.value().is_expired() {
                expired_tokens.push(token_entry.key().clone());
            }
        }

        let count = expired_tokens.len();
        for token_id in expired_tokens {
            self.search_tokens.remove(&token_id);
            debug!("移除过期搜索令牌: {token_id}");
        }

        if count > 0 {
            info!("清理了 {count} 个过期搜索令牌");
        }

        Ok(count)
    }

    /// 计算目录统计信息
    fn calculate_directory_stats(&self, path: &Path) -> Result<(u64, u64)> {
        let mut file_count = 0;
        let mut total_size = 0;

        fn visit_dir(dir: &Path, file_count: &mut u64, total_size: &mut u64) -> Result<()> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dir(&path, file_count, total_size)?;
                    } else {
                        *file_count += 1;
                        if let Ok(metadata) = entry.metadata() {
                            *total_size += metadata.len();
                        }
                    }
                }
            }
            Ok(())
        }

        visit_dir(path, &mut file_count, &mut total_size)?;
        Ok((file_count, total_size))
    }

    /// 文件路径安全处理方法
    fn validate_path_security(&self, mount_root: &Path, target_path: &Path) -> Result<()> {
        // 规范化路径
        let canonical_mount = fs::canonicalize(mount_root)
            .context("无法规范化挂载点路径")?;
        
        let canonical_target = fs::canonicalize(target_path)
            .or_else(|_| {
                // 如果目标路径不存在，检查其父目录
                if let Some(parent) = target_path.parent() {
                    if parent.exists() {
                        let canonical_parent = fs::canonicalize(parent)?;
                        if let Some(filename) = target_path.file_name() {
                            Ok(canonical_parent.join(filename))
                        } else {
                            Ok(canonical_parent)
                        }
                    } else {
                        Err(anyhow!("路径不存在且无法验证安全性"))
                    }
                } else {
                    Err(anyhow!("无效路径"))
                }
            })?;

        // 检查目标路径是否在挂载点范围内
        if !canonical_target.starts_with(&canonical_mount) {
            return Err(anyhow!("访问被拒绝：路径超出挂载点范围"));
        }

        // 检查路径中是否包含危险元素
        let path_str = canonical_target.to_string_lossy();
        if path_str.contains("..") || path_str.contains("//") {
            return Err(anyhow!("访问被拒绝：路径包含危险元素"));
        }

        Ok(())
    }

    /// 根据模式搜索文件
    fn search_files_by_pattern(&self, root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        fn search_recursive(dir: &Path, pattern: &str, files: &mut Vec<PathBuf>) -> Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    search_recursive(&path, pattern, files)?;
                } else {
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    
                    // 简单的模式匹配
                    if pattern.ends_with("*") {
                        let prefix = &pattern[..pattern.len() - 1];
                        if filename.starts_with(prefix) {
                            files.push(path);
                        }
                    } else if pattern.starts_with("*") {
                        let suffix = &pattern[1..];
                        if filename.ends_with(suffix) {
                            files.push(path);
                        }
                    } else if filename.contains(pattern) {
                        files.push(path);
                    }
                }
            }
            Ok(())
        }

        search_recursive(root, pattern, &mut files)?;
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_mount_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mount_manager = MountManager::new();

        let mount_point = MountPoint {
            id: Uuid::new_v4().to_string(),
            local_path: temp_dir.path().to_path_buf(),
            mount_name: "测试挂载点".to_string(),
            read_only: false,
            mount_time: Utc::now(),
            file_count: 0,
            total_size: 0,
        };

        let result = mount_manager.mount_directory(mount_point.clone()).await;
        assert!(result.is_ok());

        let mount_points = mount_manager.get_mount_points().await.unwrap();
        assert_eq!(mount_points.len(), 1);
        assert_eq!(mount_points[0].mount_name, "测试挂载点");
    }

    #[tokio::test]
    async fn test_search_token() {
        let mount_manager = MountManager::new();
        let mount_id = "test_mount".to_string();

        // 先创建一个挂载点
        let temp_dir = TempDir::new().unwrap();
        let mount_point = MountPoint {
            id: mount_id.clone(),
            local_path: temp_dir.path().to_path_buf(),
            mount_name: "测试挂载点".to_string(),
            read_only: false,
            mount_time: Utc::now(),
            file_count: 0,
            total_size: 0,
        };

        mount_manager.mount_directory(mount_point).await.unwrap();

        // 创建搜索令牌
        let token_id = mount_manager.create_search_token(
            mount_id,
            vec!["*.txt".to_string()],
            vec!["read".to_string()],
            3600,
        ).await.unwrap();

        // 验证令牌
        let is_authorized = mount_manager.validate_search_token(&token_id, "test.txt").await.unwrap();
        assert!(is_authorized);

        let is_not_authorized = mount_manager.validate_search_token(&token_id, "test.jpg").await.unwrap();
        assert!(!is_not_authorized);
    }

    #[tokio::test]
    async fn test_file_authorization() {
        let temp_dir = TempDir::new().unwrap();
        let mount_manager = MountManager::new();

        // 创建挂载点
        let mount_point = MountPoint {
            id: "test_mount".to_string(),
            local_path: temp_dir.path().to_path_buf(),
            mount_name: "测试挂载点".to_string(),
            read_only: false,
            mount_time: Utc::now(),
            file_count: 0,
            total_size: 0,
        };

        mount_manager.mount_directory(mount_point).await.unwrap();

        // 创建测试文件
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // 授权文件
        let auth_id = mount_manager.authorize_file(
            test_file,
            "read".to_string(),
            vec!["read".to_string()],
        ).await.unwrap();

        assert!(!auth_id.is_empty());
    }
}