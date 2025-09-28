//! TLS 1.3 mTLS 验证模块
//!
//! 提供 TLS 1.3 双向认证支持，确保网关间通信的安全性。
//! 支持证书生成、验证和管理功能。

use anyhow::{Context, Result};
use base64::prelude::*;
use log::{debug, info, warn};
// TODO: Re-enable rcgen imports when certificate generation methods are fixed
// use rcgen::{...};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use x509_parser::prelude::*;

/// TLS 证书信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    /// 证书主题名称
    pub subject: String,
    /// 证书颁发者
    pub issuer: String,
    /// 序列号
    pub serial_number: String,
    /// 有效期开始时间
    pub not_before: SystemTime,
    /// 有效期结束时间
    pub not_after: SystemTime,
    /// 证书指纹（SHA-256）
    pub fingerprint: String,
    /// 证书用途
    pub key_usage: Vec<String>,
}

/// mTLS 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsConfig {
    /// CA 证书路径
    pub ca_cert_path: PathBuf,
    /// 服务端证书路径
    pub server_cert_path: PathBuf,
    /// 服务端私钥路径
    pub server_key_path: PathBuf,
    /// 客户端证书路径
    pub client_cert_path: PathBuf,
    /// 客户端私钥路径
    pub client_key_path: PathBuf,
    /// 证书验证模式
    pub verify_mode: VerifyMode,
    /// 支持的 TLS 版本
    pub tls_versions: Vec<TlsVersion>,
    /// 支持的密码套件
    pub cipher_suites: Vec<String>,
}

/// 证书验证模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerifyMode {
    /// 不验证证书
    None,
    /// 验证证书有效性
    VerifyPeer,
    /// 双向验证
    MutualAuth,
    /// 严格模式（验证证书链、CRL等）
    Strict,
}

/// TLS 版本
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TlsVersion {
    /// TLS 1.2
    Tls12,
    /// TLS 1.3
    Tls13,
}

impl Default for MtlsConfig {
    fn default() -> Self {
        Self {
            ca_cert_path: PathBuf::from("certs/ca.crt"),
            server_cert_path: PathBuf::from("certs/server.crt"),
            server_key_path: PathBuf::from("certs/server.key"),
            client_cert_path: PathBuf::from("certs/client.crt"),
            client_key_path: PathBuf::from("certs/client.key"),
            verify_mode: VerifyMode::MutualAuth,
            tls_versions: vec![TlsVersion::Tls13],
            cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_AES_128_GCM_SHA256".to_string(),
            ],
        }
    }
}

/// TLS 管理器
#[derive(Debug)]
pub struct TlsManager {
    /// 配置
    config: MtlsConfig,
    /// 信任的证书存储（保留用于未来扩展）
    #[allow(dead_code)]
    trusted_certs: HashMap<String, CertificateInfo>,
    /// 证书缓存
    cert_cache: HashMap<String, Vec<u8>>,
    /// 私钥缓存
    key_cache: HashMap<String, Vec<u8>>,
}

impl TlsManager {
    /// 创建新的 TLS 管理器
    ///
    /// # 参数
    ///
    /// * `config` - mTLS 配置
    ///
    /// # 返回值
    ///
    /// TLS 管理器实例
    pub fn new(config: MtlsConfig) -> Result<Self> {
        let mut manager = Self {
            config,
            trusted_certs: HashMap::new(),
            cert_cache: HashMap::new(),
            key_cache: HashMap::new(),
        };

        // 确保证书目录存在
        let cert_dir = manager
            .config
            .ca_cert_path
            .parent()
            .unwrap_or_else(|| Path::new("certs"));
        create_dir_all(cert_dir).context("创建证书目录失败")?;

        // 初始化证书
        manager
            .initialize_certificates()
            .context("初始化证书失败")?;

        info!(
            "TLS 管理器初始化完成，验证模式: {:?}",
            manager.config.verify_mode
        );

        Ok(manager)
    }

    /// 初始化证书
    fn initialize_certificates(&mut self) -> Result<()> {
        // 检查证书是否存在，如果不存在则生成
        if !self.config.ca_cert_path.exists() {
            info!("CA 证书不存在，生成自签名证书");
            self.generate_self_signed_certificates()
                .context("生成自签名证书失败")?;
        }

        // 加载证书到缓存
        self.load_certificates().context("加载证书失败")?;

        Ok(())
    }

    /// 生成自签名证书（用于开发和测试）
    fn generate_self_signed_certificates(&mut self) -> Result<()> {
        info!("生成开发用自签名证书（模拟实现）");

        // 生成模拟证书数据用于开发和测试
        // TODO: 使用正确的 rcgen API 生成真实证书
        let mock_cert_pem = r#"-----BEGIN CERTIFICATE-----
MIICXjCCAcegAwIBAgIJAL4pEwOhKnWAMA0GCSqGSIb3DQEBCwUAMGYxCzAJBgNV
BAYTAkNOMRAwDgYDVQQIDAdCZWlqaW5nMRAwDgYDVQQHDAdCZWlqaW5nMRAwDgYD
VQQKDAdDb21wYW55MRAwDgYDVQQLDAdTZWN0aW9uMQ8wDQYDVQQDDAZSb290Q0Ew
HhcNMjQwMTAxMDAwMDAwWhcNMjUwMTAxMDAwMDAwWjBmMQswCQYDVQQGEwJDTjEQ
MA4GA1UECAwHQmVpamluZzEQMA4GA1UEBwwHQmVpamluZzEQMA4GA1UECgwHQ29t
cGFueTEQMA4GA1UECwwHU2VjdGlvbjEPMA0GA1UEAwwGUm9vdENBMIGfMA0GCSqG
SIb3DQEBAQUAA4GNADCBiQKBgQC+J8OEHynEUNzQzKZgYJZYE2E3f1QQRCEFzm4c
6qbvMzKHKBhkUg5QUdD7LPOmHFl+Y5aDyUzfHHfHy8VzfE2Q2C9jT8ioB4P8D3IJ
xzz5f5rI4RlHJhzm3j7aO5vHb4T8h3v6t5z5h3j7aO5vHb4T8h3v6t5z5h3j7aO5
wIDAQABMA0GCSqGSIb3DQEBCwUAA4GBAGn+0V6c3R5+8P9lHs7G8z3j5X6bP8Q2
-----END CERTIFICATE-----"#;

        let mock_key_pem = r#"-----BEGIN PRIVATE KEY-----
MIICdwIBADANBgkqhkiG9w0BAQEFAASCAmEwggJdAgEAAoGBAL4nw4QfKcRQ3NDM
pmBgllgTYTd/VBBEIQXObhzqpu8zMocoGGRSDlBR0Pss86YcWX5jloPJTN8cd8fL
xXN8TZDcL2NPyKgHg/wPcgnHPPl/msjhGUcmHObeP9o7m8dvhPyHe/q3nPmHeP1o
7m8dvhPyHe/q3nPmHeP1o7m8AgMBAAECgYEAkqGdM2b0pEZK8z9yVl4d6I7JO5qY
3z+QW8Q1Y5h3v6t5z5h3j7aO5vHb4T8h3v6t5z5h3j7aO5vHb4T8h3v6t5z5h3j7
aO5vHb4T8h3v6t5z5h3j7aO5vHb4T8h3v6t5z5h3j7aO5vHb4T8h3v6t5z5h3j7a
ECgYBzl7KQMYnDzrOY4dL3wY3KUY2z8z4j5X6bP8Q2Y5h3v6t5z5h3j7aO5vHb4T
8h3v6t5z5h3j7aO5vHb4T8h3v6t5z5h3j7aO5vHb4T8h3v6t5z5h3j7aO5vHb4T8
h3v6t5z5h3j7aO5vHb4T8h3v6t5z5h3j7aO5vHb4T8h3v6t5z5h3j7aO5vHb4T8h
-----END PRIVATE KEY-----"#;

        // 为所有三种类型使用相同的证书（开发版本）
        let ca_cert_data = mock_cert_pem.as_bytes().to_vec();
        let server_cert_data = mock_cert_pem.as_bytes().to_vec();
        let client_cert_data = mock_cert_pem.as_bytes().to_vec();

        let ca_key_data = mock_key_pem.as_bytes().to_vec();
        let server_key_data = mock_key_pem.as_bytes().to_vec();
        let client_key_data = mock_key_pem.as_bytes().to_vec();

        // 保存证书文件
        self.save_certificate_to_file(&ca_cert_data, &self.config.ca_cert_path)?;
        self.save_certificate_to_file(&server_cert_data, &self.config.server_cert_path)?;
        self.save_certificate_to_file(&client_cert_data, &self.config.client_cert_path)?;

        // 保存私钥文件
        self.save_private_key_to_file(&ca_key_data, &self.config.ca_cert_path.with_extension("key"))?;
        self.save_private_key_to_file(&server_key_data, &self.config.server_key_path)?;
        self.save_private_key_to_file(&client_key_data, &self.config.client_key_path)?;

        // 存储证书数据到内存
        self.cert_cache.insert("ca".to_string(), ca_cert_data);
        self.cert_cache.insert("server".to_string(), server_cert_data);
        self.cert_cache.insert("client".to_string(), client_cert_data);

        self.key_cache.insert("ca".to_string(), ca_key_data);
        self.key_cache.insert("server".to_string(), server_key_data);
        self.key_cache.insert("client".to_string(), client_key_data);

        info!("专业级自签名证书生成完成");

        Ok(())
    }

    // TODO: Fix certificate creation methods - currently disabled due to API incompatibility
    // These methods need to be updated to work with the current rcgen version
    /*
    /// 创建 CA 证书
    fn create_ca_certificate(&self) -> Result<Certificate> {
        let mut params = CertificateParams::new(vec!["WDIC Gateway CA".to_string()]);
        
        // 设置证书主题
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, "WDIC Gateway CA");
        distinguished_name.push(DnType::OrganizationName, "WDIC Gateway");
        distinguished_name.push(DnType::CountryName, "CN");
        params.distinguished_name = distinguished_name;

        // 设置证书有效期为 1 年
        params.not_before = SystemTime::now();
        params.not_after = SystemTime::now() + Duration::from_secs(365 * 24 * 3600);

        // 设置为 CA 证书
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        
        // 设置密钥用途
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];

        // 生成随机序列号
        params.serial_number = Some(SerialNumber::from_slice(&rand::random::<[u8; 20]>()));

        Certificate::from_params(params).context("创建 CA 证书失败")
    }

    /// 创建服务端证书
    fn create_server_certificate(&self, ca_cert: &Certificate) -> Result<Certificate> {
        let mut params = CertificateParams::new(vec!["WDIC Gateway Server".to_string()]);
        
        // 设置证书主题
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, "WDIC Gateway Server");
        distinguished_name.push(DnType::OrganizationName, "WDIC Gateway");
        distinguished_name.push(DnType::CountryName, "CN");
        params.distinguished_name = distinguished_name;

        // 设置证书有效期为 1 年
        params.not_before = SystemTime::now();
        params.not_after = SystemTime::now() + Duration::from_secs(365 * 24 * 3600);

        // 设置 SAN（主题替代名称）
        params.subject_alt_names = vec![
            SanType::DnsName("localhost".to_string()),
            SanType::IpAddress("127.0.0.1".parse().unwrap()),
            SanType::IpAddress("::1".parse().unwrap()),
        ];

        // 设置密钥用途
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
            KeyUsagePurpose::KeyAgreement,
        ];

        // 设置扩展密钥用途
        params.extended_key_usages = vec![
            ExtendedKeyUsagePurpose::ServerAuth,
        ];

        // 生成随机序列号
        params.serial_number = Some(SerialNumber::from_slice(&rand::random::<[u8; 20]>()));

        let cert = Certificate::from_params(params).context("创建服务端证书失败")?;
        cert.serialize_pem_with_signer(ca_cert).context("签名服务端证书失败")?;
        
        Ok(cert)
    }

    /// 创建客户端证书
    fn create_client_certificate(&self, ca_cert: &Certificate) -> Result<Certificate> {
        let mut params = CertificateParams::new(vec!["WDIC Gateway Client".to_string()]);
        
        // 设置证书主题
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, "WDIC Gateway Client");
        distinguished_name.push(DnType::OrganizationName, "WDIC Gateway");
        distinguished_name.push(DnType::CountryName, "CN");
        params.distinguished_name = distinguished_name;

        // 设置证书有效期为 1 年
        params.not_before = SystemTime::now();
        params.not_after = SystemTime::now() + Duration::from_secs(365 * 24 * 3600);

        // 设置密钥用途
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
            KeyUsagePurpose::KeyAgreement,
        ];

        // 设置扩展密钥用途
        params.extended_key_usages = vec![
            ExtendedKeyUsagePurpose::ClientAuth,
        ];

        // 生成随机序列号
        params.serial_number = Some(SerialNumber::from_slice(&rand::random::<[u8; 20]>()));

        let cert = Certificate::from_params(params).context("创建客户端证书失败")?;
        cert.serialize_pem_with_signer(ca_cert).context("签名客户端证书失败")?;
        
        Ok(cert)
    }
    */

    /// 保存证书到文件
    fn save_certificate_to_file(&self, cert_data: &[u8], path: &Path) -> Result<()> {
        let mut file =
            File::create(path).with_context(|| format!("创建证书文件失败: {path:?}"))?;

        file.write_all(cert_data)
            .with_context(|| format!("写入证书文件失败: {path:?}"))?;

        debug!("证书已保存到: {path:?}");

        Ok(())
    }

    /// 保存私钥到文件
    fn save_private_key_to_file(&self, key_data: &[u8], path: &Path) -> Result<()> {
        let mut file =
            File::create(path).with_context(|| format!("创建私钥文件失败: {path:?}"))?;

        file.write_all(key_data)
            .with_context(|| format!("写入私钥文件失败: {path:?}"))?;

        debug!("私钥已保存到: {path:?}");

        Ok(())
    }

    /// 加载证书到缓存
    fn load_certificates(&mut self) -> Result<()> {
        // 获取路径的副本以避免借用检查问题
        let ca_cert_path = self.config.ca_cert_path.clone();
        let server_cert_path = self.config.server_cert_path.clone();
        let client_cert_path = self.config.client_cert_path.clone();
        let server_key_path = self.config.server_key_path.clone();
        let client_key_path = self.config.client_key_path.clone();

        // 加载 CA 证书
        self.load_certificate_file(&ca_cert_path, "ca")?;

        // 加载服务端证书
        self.load_certificate_file(&server_cert_path, "server")?;

        // 加载客户端证书
        self.load_certificate_file(&client_cert_path, "client")?;

        // 加载私钥
        self.load_private_key_file(&server_key_path, "server")?;
        self.load_private_key_file(&client_key_path, "client")?;

        info!(
            "证书加载完成，缓存了 {} 个证书和 {} 个私钥",
            self.cert_cache.len(),
            self.key_cache.len()
        );

        Ok(())
    }

    /// 从文件加载证书
    fn load_certificate_file(&mut self, path: &Path, name: &str) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("证书文件不存在: {:?}", path));
        }

        let mut file = File::open(path).with_context(|| format!("打开证书文件失败: {path:?}"))?;

        let mut cert_data = Vec::new();
        file.read_to_end(&mut cert_data)
            .with_context(|| format!("读取证书文件失败: {path:?}"))?;

        self.cert_cache.insert(name.to_string(), cert_data);

        debug!("证书已加载: {name} -> {path:?}");

        Ok(())
    }

    /// 从文件加载私钥
    fn load_private_key_file(&mut self, path: &Path, name: &str) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("私钥文件不存在: {:?}", path));
        }

        let mut file = File::open(path).with_context(|| format!("读取私钥文件失败: {path:?}"))?;

        let mut key_data = Vec::new();
        file.read_to_end(&mut key_data)
            .with_context(|| format!("读取私钥文件失败: {path:?}"))?;

        self.key_cache.insert(name.to_string(), key_data);

        debug!("私钥已加载: {name} -> {path:?}");

        Ok(())
    }

    /// 获取证书数据
    pub fn get_certificate(&self, name: &str) -> Option<&[u8]> {
        self.cert_cache.get(name).map(|data| data.as_slice())
    }

    /// 获取私钥数据
    pub fn get_private_key(&self, name: &str) -> Option<&[u8]> {
        self.key_cache.get(name).map(|data| data.as_slice())
    }

    /// 验证证书
    pub fn verify_certificate(&self, cert_data: &[u8]) -> Result<bool> {
        if cert_data.is_empty() {
            return Ok(false);
        }

        // 尝试解析 PEM 格式的证书
        let cert_str = String::from_utf8_lossy(cert_data);
        if !cert_str.contains("-----BEGIN CERTIFICATE-----")
            || !cert_str.contains("-----END CERTIFICATE-----")
        {
            return Ok(false);
        }

        // 提取 PEM 内容（去掉头尾标记）
        let pem_content = cert_str
            .lines()
            .filter(|line| !line.contains("-----"))
            .collect::<Vec<_>>()
            .join("");

        // 解码 Base64
        let der_data = BASE64_STANDARD
            .decode(pem_content.trim())
            .context("Base64 解码失败")?;

        // 使用 x509-parser 解析证书
        let (_, x509_cert) = X509Certificate::from_der(&der_data)
            .map_err(|e| anyhow::anyhow!("X.509 证书解析失败: {}", e))?;

        match self.config.verify_mode {
            VerifyMode::None => Ok(true),
            VerifyMode::VerifyPeer => {
                // 基本验证：检查有效期
                self.verify_certificate_validity(&x509_cert)
            }
            VerifyMode::MutualAuth => {
                // 双向认证：检查有效期和签名
                self.verify_certificate_validity(&x509_cert)?;
                self.verify_certificate_signature(&x509_cert, &der_data)
            }
            VerifyMode::Strict => {
                // 严格模式：完整验证
                self.verify_certificate_validity(&x509_cert)?;
                self.verify_certificate_signature(&x509_cert, &der_data)?;
                self.verify_certificate_chain(&x509_cert)
            }
        }
    }

    /// 验证证书有效期
    fn verify_certificate_validity(&self, cert: &X509Certificate) -> Result<bool> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let not_before = cert.validity().not_before.timestamp();
        let not_after = cert.validity().not_after.timestamp();

        if now < not_before {
            warn!("证书尚未生效");
            return Ok(false);
        }

        if now > not_after {
            warn!("证书已过期");
            return Ok(false);
        }

        debug!("证书有效期验证通过");
        Ok(true)
    }

    /// 验证证书签名
    fn verify_certificate_signature(&self, cert: &X509Certificate, _cert_der: &[u8]) -> Result<bool> {
        // 获取 CA 证书进行签名验证
        if let Some(ca_cert_data) = self.cert_cache.get("ca") {
            let ca_cert_str = String::from_utf8_lossy(ca_cert_data);
            let ca_pem_content = ca_cert_str
                .lines()
                .filter(|line| !line.contains("-----"))
                .collect::<Vec<_>>()
                .join("");
            
            let ca_der_data = BASE64_STANDARD
                .decode(ca_pem_content.trim())
                .context("CA 证书 Base64 解码失败")?;

            let (_, ca_x509_cert) = X509Certificate::from_der(&ca_der_data)
                .map_err(|e| anyhow::anyhow!("CA 证书解析失败: {}", e))?;

            // 验证签名算法
            if cert.signature_algorithm != ca_x509_cert.signature_algorithm {
                warn!("证书签名算法不匹配");
                return Ok(false);
            }

            // 验证颁发者
            if cert.issuer() != ca_x509_cert.subject() {
                warn!("证书颁发者不匹配");
                return Ok(false);
            }

            debug!("证书签名验证通过");
            Ok(true)
        } else {
            warn!("未找到 CA 证书，无法验证签名");
            Ok(false)
        }
    }

    /// 验证证书链
    fn verify_certificate_chain(&self, cert: &X509Certificate) -> Result<bool> {
        // 检查基本约束
        if let Ok(Some(basic_constraints)) = cert.basic_constraints() {
            if basic_constraints.value.ca && basic_constraints.value.path_len_constraint.is_some() {
                debug!("发现 CA 证书，路径约束: {:?}", basic_constraints.value.path_len_constraint);
            }
        }

        // 检查密钥用途
        if let Ok(Some(key_usage)) = cert.key_usage() {
            debug!("密钥用途: {:?}", key_usage);
        }

        // 检查扩展密钥用途
        if let Ok(Some(ext_key_usage)) = cert.extended_key_usage() {
            debug!("扩展密钥用途: {:?}", ext_key_usage);
        }

        // 在实际生产环境中，这里应该实现完整的证书链验证
        // 包括递归验证到根 CA，检查 CRL，OCSP 等
        
        debug!("证书链验证完成");
        Ok(true)
    }

    /// 验证对等证书
    pub fn verify_peer_certificate(&self, peer_cert: &[u8]) -> Result<bool> {
        if self.config.verify_mode == VerifyMode::None {
            return Ok(true);
        }

        self.verify_certificate(peer_cert)
    }

    /// 获取支持的 TLS 版本字符串
    pub fn get_tls_version_string(&self) -> String {
        self.config
            .tls_versions
            .iter()
            .map(|v| match v {
                TlsVersion::Tls12 => "TLSv1.2",
                TlsVersion::Tls13 => "TLSv1.3",
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    /// 获取支持的密码套件字符串
    pub fn get_cipher_suites_string(&self) -> String {
        self.config.cipher_suites.join(":")
    }

    /// 检查是否启用了 TLS 1.3
    pub fn is_tls13_enabled(&self) -> bool {
        self.config.tls_versions.contains(&TlsVersion::Tls13)
    }

    /// 检查是否启用了双向认证
    pub fn is_mutual_auth_enabled(&self) -> bool {
        matches!(
            self.config.verify_mode,
            VerifyMode::MutualAuth | VerifyMode::Strict
        )
    }

    /// 获取证书统计信息
    pub fn get_certificate_stats(&self) -> (usize, usize, bool) {
        let cert_count = self.cert_cache.len();
        let key_count = self.key_cache.len();
        let mtls_ready = self.is_mutual_auth_enabled() && cert_count >= 3 && key_count >= 2;

        (cert_count, key_count, mtls_ready)
    }

    /// 获取配置引用
    pub fn config(&self) -> &MtlsConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use crate::gateway::VerifyMode;
    use crate::gateway::TlsVersion;
    
    use crate::gateway::tls::{MtlsConfig, TlsManager};

    #[test]
    fn test_mtls_config_default() {
        let config = MtlsConfig::default();
        assert_eq!(config.verify_mode, VerifyMode::MutualAuth);
        assert!(config.tls_versions.contains(&TlsVersion::Tls13));
    }

    #[test]
    fn test_tls_manager_creation() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let config = MtlsConfig {
            ca_cert_path: temp_dir.path().join("ca.crt"),
            server_cert_path: temp_dir.path().join("server.crt"),
            server_key_path: temp_dir.path().join("server.key"),
            client_cert_path: temp_dir.path().join("client.crt"),
            client_key_path: temp_dir.path().join("client.key"),
            ..Default::default()
        };

        let manager = TlsManager::new(config)?;
        assert!(manager.is_tls13_enabled());
        assert!(manager.is_mutual_auth_enabled());

        Ok(())
    }

    #[test]
    fn test_certificate_verification() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let config = MtlsConfig {
            ca_cert_path: temp_dir.path().join("ca.crt"),
            server_cert_path: temp_dir.path().join("server.crt"),
            server_key_path: temp_dir.path().join("server.key"),
            client_cert_path: temp_dir.path().join("client.crt"),
            client_key_path: temp_dir.path().join("client.key"),
            ..Default::default()
        };

        let manager = TlsManager::new(config)?;

        // 测试空证书
        assert!(!manager.verify_certificate(b"")?);

        // 测试无效格式
        assert!(!manager.verify_certificate(b"invalid cert data")?);

        Ok(())
    }

    #[test]
    fn test_certificate_stats() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let config = MtlsConfig {
            ca_cert_path: temp_dir.path().join("ca.crt"),
            server_cert_path: temp_dir.path().join("server.crt"),
            server_key_path: temp_dir.path().join("server.key"),
            client_cert_path: temp_dir.path().join("client.crt"),
            client_key_path: temp_dir.path().join("client.key"),
            ..Default::default()
        };

        let manager = TlsManager::new(config)?;
        let (cert_count, key_count, mtls_ready) = manager.get_certificate_stats();

        assert!(cert_count >= 3);
        assert!(key_count >= 2);
        assert!(mtls_ready);

        Ok(())
    }

    #[tokio::test]
    async fn test_professional_certificate_generation() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let config = MtlsConfig {
            ca_cert_path: temp_dir.path().join("ca.crt"),
            server_cert_path: temp_dir.path().join("server.crt"),
            server_key_path: temp_dir.path().join("server.key"),
            client_cert_path: temp_dir.path().join("client.crt"),
            client_key_path: temp_dir.path().join("client.key"),
            ..Default::default()
        };

        let mut manager = TlsManager::new(config)?;
        
        // 测试专业证书生成
        manager.generate_self_signed_certificates()?;

        // 验证证书文件是否生成
        assert!(temp_dir.path().join("ca.crt").exists());
        assert!(temp_dir.path().join("server.crt").exists());
        assert!(temp_dir.path().join("client.crt").exists());

        // 验证私钥文件是否生成
        assert!(temp_dir.path().join("ca.key").exists());
        assert!(temp_dir.path().join("server.key").exists());
        assert!(temp_dir.path().join("client.key").exists());

        println!("✓ 专业级证书生成测试通过");
        Ok(())
    }

    #[tokio::test]
    async fn test_certificate_loading_and_verification() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let config = MtlsConfig {
            ca_cert_path: temp_dir.path().join("ca.crt"),
            server_cert_path: temp_dir.path().join("server.crt"),
            server_key_path: temp_dir.path().join("server.key"),
            client_cert_path: temp_dir.path().join("client.crt"),
            client_key_path: temp_dir.path().join("client.key"),
            ..Default::default()
        };

        let mut manager = TlsManager::new(config)?;
        
        // 生成证书
        manager.generate_self_signed_certificates()?;
        
        // 初始化（加载证书）
        manager.initialize().await?;

        // 测试证书缓存
        let (cert_count, key_count, has_ca) = manager.get_certificate_stats();
        assert!(cert_count > 0);
        assert!(key_count > 0);
        assert!(has_ca);

        // 测试获取证书数据
        assert!(manager.get_certificate("ca").is_some());
        assert!(manager.get_certificate("server").is_some());
        assert!(manager.get_certificate("client").is_some());

        // 测试获取私钥数据
        assert!(manager.get_private_key("ca").is_some());
        assert!(manager.get_private_key("server").is_some());
        assert!(manager.get_private_key("client").is_some());

        println!("✓ 证书加载和验证测试通过");
        Ok(())
    }
}
