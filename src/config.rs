use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub remote: RemoteConfig,
    pub notification: NotificationConfig,
    pub gewe_cli: GeweCliConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteConfig {
    /// 全局远程模式开关
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// 通知渠道：wechat, telegram, dingtalk 等
    pub channel: String,

    /// 微信 ID
    pub wxid: String,

    /// 监听地址
    pub listen: String,

    /// Transcript 展示域名
    #[serde(default)]
    pub transcript_domain: String,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            channel: "wechat".to_string(),
            wxid: String::new(),
            listen: String::new(),
            transcript_domain: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeweCliConfig {
    /// gewe-cli 命令路径
    #[serde(default = "default_gewe_cli_command")]
    pub command: String,

    /// 超时设置（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_gewe_cli_command() -> String {
    "gewe-cli".to_string()
}

fn default_timeout() -> u64 {
    0  // 0 表示无限等待（不传 --timeout 给 gewe-cli）
}

impl Default for Config {
    fn default() -> Self {
        Self {
            remote: RemoteConfig { enabled: false },
            notification: NotificationConfig::default(),
            gewe_cli: GeweCliConfig {
                command: default_gewe_cli_command(),
                timeout: default_timeout(),
            },
        }
    }
}

pub struct ConfigManager {
    config_dir: PathBuf,
    config_file: PathBuf,
    lock_file: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))?
            .join(".gewe-cc");

        Ok(Self {
            config_file: config_dir.join("config.toml"),
            lock_file: config_dir.join("remote.lock"),
            config_dir,
        })
    }

    /// 加载配置
    pub fn load(&self) -> Result<Config> {
        if !self.config_file.exists() {
            anyhow::bail!(
                "配置文件不存在: {}\n请先运行: gewe-cc init",
                self.config_file.display()
            );
        }

        let content = fs::read_to_string(&self.config_file)
            .context("读取配置文件失败")?;

        toml::from_str(&content).context("解析配置文件失败")
    }

    /// 保存配置
    pub fn save(&self, config: &Config) -> Result<()> {
        fs::create_dir_all(&self.config_dir).context("创建配置目录失败")?;

        let content = toml::to_string_pretty(config).context("序列化配置失败")?;

        fs::write(&self.config_file, content).context("写入配置文件失败")?;

        Ok(())
    }

    /// 检查远程模式是否启用
    pub fn is_remote_enabled(&self) -> bool {
        // 优先检查 lock 文件
        if self.lock_file.exists() {
            return true;
        }

        // 备选：读取配置文件
        if let Ok(config) = self.load() {
            return config.remote.enabled;
        }

        false
    }

    /// 启用远程模式
    pub fn enable_remote(&self) -> Result<()> {
        // 1. 更新配置文件
        let mut config = self.load().unwrap_or_default();
        config.remote.enabled = true;
        self.save(&config)?;

        // 2. 创建 lock 文件
        fs::write(&self.lock_file, "").context("创建 lock 文件失败")?;

        Ok(())
    }

    /// 禁用远程模式
    pub fn disable_remote(&self) -> Result<()> {
        // 1. 更新配置文件
        if let Ok(mut config) = self.load() {
            config.remote.enabled = false;
            self.save(&config)?;
        }

        // 2. 删除 lock 文件
        if self.lock_file.exists() {
            fs::remove_file(&self.lock_file).context("删除 lock 文件失败")?;
        }

        Ok(())
    }

    /// 获取配置目录路径
    /// 用于 config 命令和其他需要显示配置位置的场景
    #[allow(dead_code)]
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// 获取配置文件路径
    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    /// 更新微信配置
    pub fn update_notification(&self, wxid: Option<String>, listen: Option<String>, transcript_domain: Option<String>) -> Result<()> {
        let mut config = self.load()?;

        if let Some(wxid) = wxid {
            config.notification.wxid = wxid;
        }

        if let Some(listen) = listen {
            config.notification.listen = listen;
        }

        if let Some(domain) = transcript_domain {
            config.notification.transcript_domain = domain;
        }

        self.save(&config)?;
        Ok(())
    }

    /// 禁用指定会话的远程模式（仅该会话）
    pub fn disable_session(&self, session_id: &str) -> Result<()> {
        if session_id.trim().is_empty() {
            return Ok(());
        }

        let mut sessions = self.load_disabled_sessions()?;
        sessions.insert(session_id.to_string());
        self.save_disabled_sessions(&sessions)?;
        Ok(())
    }

    /// 判断会话是否已被禁用
    pub fn is_session_disabled(&self, session_id: &str) -> bool {
        if session_id.trim().is_empty() {
            return false;
        }

        self.load_disabled_sessions()
            .map(|sessions| sessions.contains(session_id))
            .unwrap_or(false)
    }

    fn session_disabled_file(&self) -> PathBuf {
        self.config_dir.join("session_disabled.json")
    }

    fn load_disabled_sessions(&self) -> Result<HashSet<String>> {
        let file_path = self.session_disabled_file();
        if !file_path.exists() {
            return Ok(HashSet::new());
        }

        let content = fs::read_to_string(&file_path)
            .context("读取会话禁用列表失败")?;
        let sessions: HashSet<String> = serde_json::from_str(&content)
            .unwrap_or_default();
        Ok(sessions)
    }

    fn save_disabled_sessions(&self, sessions: &HashSet<String>) -> Result<()> {
        fs::create_dir_all(&self.config_dir)
            .context("创建配置目录失败")?;
        let content = serde_json::to_string_pretty(sessions)
            .context("序列化会话禁用列表失败")?;
        fs::write(self.session_disabled_file(), content)
            .context("写入会话禁用列表失败")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.remote.enabled);
        assert_eq!(config.notification.channel, "wechat");
        assert_eq!(config.gewe_cli.timeout, 0);  // 默认无限等待
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[remote]"));
        assert!(toml_str.contains("[notification]"));
        assert!(toml_str.contains("[gewe_cli]"));
    }

    #[test]
    fn test_update_notification() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().to_path_buf();
        let config_file = config_dir.join("config.toml");

        // 创建初始配置
        let initial_config = Config::default();
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            &config_file,
            toml::to_string(&initial_config).unwrap(),
        )
        .unwrap();

        // 模拟 ConfigManager（这里需要创建临时实例）
        let mgr = ConfigManager {
            config_dir: config_dir.clone(),
            config_file: config_file.clone(),
            lock_file: config_dir.join("remote.lock"),
        };

        // 测试只更新 wxid
        mgr.update_notification(Some("new_wxid".to_string()), None, None)
            .unwrap();
        let config = mgr.load().unwrap();
        assert_eq!(config.notification.wxid, "new_wxid");

        // 测试只更新 listen
        mgr.update_notification(None, Some("127.0.0.1:8080".to_string()), None)
            .unwrap();
        let config = mgr.load().unwrap();
        assert_eq!(config.notification.listen, "127.0.0.1:8080");
        assert_eq!(config.notification.wxid, "new_wxid"); // wxid 保持不变

        // 测试同时更新两个
        mgr.update_notification(
            Some("another_wxid".to_string()),
            Some("0.0.0.0:9999".to_string()),
            None,
        )
        .unwrap();
        let config = mgr.load().unwrap();
        assert_eq!(config.notification.wxid, "another_wxid");
        assert_eq!(config.notification.listen, "0.0.0.0:9999");
    }
}
