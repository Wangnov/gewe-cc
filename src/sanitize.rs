/// 脱敏工具模块
///
/// 用于对敏感信息进行脱敏处理，保护用户隐私

/// 脱敏 wxid（微信ID）
///
/// # 规则
/// - 保留 `wxid_` 前缀
/// - 保留后 3 位字符
/// - 中间用 `***` 替代
///
/// # 示例
/// ```
/// use gewe_cc::sanitize::sanitize_wxid;
///
/// assert_eq!(sanitize_wxid("wxid_mly499mvz23o21"), "wxid_***o21");
/// assert_eq!(sanitize_wxid("wxid_abc"), "wxid_***abc");
/// assert_eq!(sanitize_wxid(""), "");
/// ```
pub fn sanitize_wxid(wxid: &str) -> String {
    if wxid.is_empty() {
        return String::new();
    }

    // 如果是 wxid_ 格式
    if let Some(id_part) = wxid.strip_prefix("wxid_") {
        if id_part.len() < 3 {
            // 如果 ID 部分太短（小于 3 位），直接保留
            return wxid.to_string();
        }
        // 保留后 3 位
        let suffix = &id_part[id_part.len() - 3..];
        return format!("wxid_***{}", suffix);
    }

    // 其他格式，保留前 6 位和后 3 位
    if wxid.len() <= 9 {
        return wxid.to_string();
    }
    let prefix = &wxid[..6];
    let suffix = &wxid[wxid.len() - 3..];
    format!("{}***{}", prefix, suffix)
}

/// 脱敏监听地址
///
/// # 规则
/// - 保留 `0.0.0.0` 和 `127.0.0.1` 以及 `localhost` 不脱敏
/// - 其他 IP 地址脱敏为 `*.*.*.*`
/// - 保留端口号
///
/// # 示例
/// ```
/// use gewe_cc::sanitize::sanitize_listen_addr;
///
/// assert_eq!(sanitize_listen_addr("0.0.0.0:4399"), "0.0.0.0:4399");
/// assert_eq!(sanitize_listen_addr("127.0.0.1:4399"), "127.0.0.1:4399");
/// assert_eq!(sanitize_listen_addr("localhost:4399"), "localhost:4399");
/// assert_eq!(sanitize_listen_addr("192.168.1.100:4399"), "*.*.*.*:4399");
/// assert_eq!(sanitize_listen_addr("10.0.0.5:8080"), "*.*.*.*:8080");
/// ```
pub fn sanitize_listen_addr(addr: &str) -> String {
    // 先检查是否是本地地址（不需要端口分离）
    if addr.starts_with("0.0.0.0")
        || addr.starts_with("127.0.0.1")
        || addr.starts_with("localhost")
        || addr.starts_with("::1")
    {
        return addr.to_string();
    }

    // 从右边分离 IP 和端口（支持 IPv6）
    if let Some((ip, port)) = addr.rsplit_once(':') {
        // 再次检查 IP 部分（去除端口后）
        if ip == "0.0.0.0" || ip == "127.0.0.1" || ip == "localhost" || ip == "::1" {
            return addr.to_string();
        }
        // 非本地地址，脱敏 IP
        return format!("*.*.*.*:{}", port);
    }

    // 没有端口号的情况
    if addr == "0.0.0.0" || addr == "127.0.0.1" || addr == "localhost" || addr == "::1" {
        return addr.to_string();
    }

    // 非本地地址
    "*.*.*.*".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_wxid() {
        assert_eq!(sanitize_wxid("wxid_mly499mvz23o21"), "wxid_***o21");
        assert_eq!(sanitize_wxid("wxid_abc"), "wxid_***abc");
        assert_eq!(sanitize_wxid("wxid_abcdefgh"), "wxid_***fgh");
        assert_eq!(sanitize_wxid(""), "");

        // 非 wxid_ 格式
        assert_eq!(sanitize_wxid("user123456789"), "user12***789");
        assert_eq!(sanitize_wxid("short"), "short");
    }

    #[test]
    fn test_sanitize_listen_addr() {
        // 本地地址不脱敏
        assert_eq!(sanitize_listen_addr("0.0.0.0:4399"), "0.0.0.0:4399");
        assert_eq!(sanitize_listen_addr("127.0.0.1:4399"), "127.0.0.1:4399");
        assert_eq!(sanitize_listen_addr("localhost:4399"), "localhost:4399");
        assert_eq!(sanitize_listen_addr("::1:4399"), "::1:4399");

        // 非本地地址脱敏
        assert_eq!(sanitize_listen_addr("192.168.1.100:4399"), "*.*.*.*:4399");
        assert_eq!(sanitize_listen_addr("10.0.0.5:8080"), "*.*.*.*:8080");
        assert_eq!(sanitize_listen_addr("172.16.0.1:3000"), "*.*.*.*:3000");

        // 没有端口号
        assert_eq!(sanitize_listen_addr("0.0.0.0"), "0.0.0.0");
        assert_eq!(sanitize_listen_addr("192.168.1.1"), "*.*.*.*");
    }
}
