# gewe-cc

> Claude Code 远程协作模式命令行工具

[![Crates.io](https://img.shields.io/crates/v/gewe-cc.svg)](https://crates.io/crates/gewe-cc)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

gewe-cc 是一个命令行工具，允许你通过微信远程控制 Claude Code 的工作流程。当任务完成时，自动发送微信通知并等待你的下一个指令，实现真正的远程协作。

## ✨ 特性

- 🔄 **全局远程模式**：一键启用/禁用远程模式
- 📱 **微信集成**：通过 gewe-cli 发送通知和接收指令
- 🎯 **智能循环**：根据回复自动继续工作或停止
- ⚡ **零 Python 依赖**：纯 Rust 实现，跨平台支持
- 🛡️ **会话隔离**：基于 session_id 的状态管理
- 📦 **易于安装**：cargo install 一键安装

## 📦 安装

### 使用 Cargo 安装（推荐）

```bash
cargo install gewe-cc
```

### 从源码编译

```bash
git clone https://github.com/wangnov/gewe-cc.git
cd gewe-cc
cargo build --release
```

### 使用安装脚本

查看 [GitHub Releases](https://github.com/wangnov/gewe-cc/releases) 获取最新的安装脚本。

**Linux/macOS:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/wangnov/gewe-cc/releases/latest/download/gewe-cc-installer.sh | sh
```

**Windows (PowerShell):**
```powershell
powershell -c "irm https://github.com/wangnov/gewe-cc/releases/latest/download/gewe-cc-installer.ps1 | iex"
```

## 🚀 快速开始

### 1. 安装依赖

gewe-cc 需要以下依赖：

- **gewe-cli**: 微信消息收发工具
  ```bash
  cargo install gewe-cli
  # 或
  brew install gewe-cli
  ```

- **Claude Code**: Anthropic 官方 CLI
  ```bash
  curl -fsSL https://install.claudecode.com | sh
  ```

### 2. 初始化

```bash
gewe-cc init
```

这将：
- ✅ 检查所有依赖是否已安装
- ✅ 生成配置文件 `~/.gewe-cc/config.toml`
- ✅ 提供详细的安装指引（如有缺失依赖）

### 3. 安装 Claude Code Plugin

**方式 1：从 GitHub 安装（推荐）**

```bash
# 1. 添加 gewe-cc marketplace
claude plugin marketplace add wangnov/gewe-cc

# 2. 安装 plugin
claude plugin install gewe-cc
```

**方式 2：本地安装（开发模式）**

```bash
# 1. 克隆仓库
git clone https://github.com/wangnov/gewe-cc.git
cd gewe-cc

# 2. 添加本地 marketplace
claude plugin marketplace add ./plugin

# 3. 安装 plugin
claude plugin install gewe-cc
```

**验证安装**

```bash
# 查看已安装的插件
claude plugin list

# 应该能看到 gewe-cc 插件
```

### 4. 启用远程模式

```bash
gewe-cc on
```

或在 Claude Code 中输入：

**会话级别命令**（只影响当前会话）：
```
>remote-on       # 启用当前会话的远程模式
>remote-off      # 禁用当前会话的远程模式
>remote-status   # 查看全局远程模式状态
```

**全局级别命令**（影响所有会话）：
```bash
gewe-cc on       # 全局启用远程模式
gewe-cc off      # 全局禁用远程模式
```

### 5. 开始工作

```bash
claude
```

在 Claude Code 中：
```
创建一个文件 test.txt
```

任务完成后会自动：
- 📤 发送微信通知
- ⏳ 等待你的回复
- 🔄 根据回复继续工作或停止

### 6. 远程控制

通过微信回复：
- `添加单元测试` → Claude 继续工作
- `优化性能` → Claude 继续工作
- `停止` → 结束远程模式

## 📖 命令

### gewe-cc init

初始化环境（检查依赖、生成配置）

```bash
gewe-cc init
```

### gewe-cc on

启用全局远程模式

```bash
gewe-cc on
```

### gewe-cc off

禁用全局远程模式，或仅关闭当前会话

```bash
# 禁用全局远程模式（所有任务不再进入远程控制）
gewe-cc off

# 仅关闭当前会话（保持全局远程模式，后续任务仍会进入远程控制）
gewe-cc off --session-id <会话ID>
```

会话 ID 可以从 Stop Hook 的提示中获取。

### gewe-cc status

查看远程模式状态

```bash
gewe-cc status
```

### gewe-cc config

查看或修改配置

```bash
# 查看当前配置
gewe-cc config

# 修改微信 ID
gewe-cc config --wxid wxid_new_value

# 修改监听地址
gewe-cc config --listen 0.0.0.0:5000

# 同时修改多个配置
gewe-cc config --wxid wxid_new --listen 0.0.0.0:5000
```

### gewe-cc wait-reply

发送消息并等待回复（自动使用配置文件中的 wxid 和 listen）

```bash
# 使用配置文件中的默认值
gewe-cc wait-reply -M "任务完成了"

# 临时覆盖目标微信
gewe-cc wait-reply -M "测试消息" --to-wxid wxid_test

# 设置超时（秒）
gewe-cc wait-reply -M "需要回复" --timeout 60

# 完整示例
gewe-cc wait-reply -M "【Claude Code】任务完成" --to-wxid wxid_xxx --listen 0.0.0.0:4399 --timeout 300
```

### gewe-cc notify

发送通知（不等待回复）

```bash
# 使用配置文件中的默认值
gewe-cc notify -M "构建成功"

# 临时覆盖目标微信
gewe-cc notify -M "部署完成" --to-wxid wxid_ops
```

### gewe-cc hook (内部命令)

处理 Claude Code Hook 事件（由 plugin 调用，非用户命令）

```bash
gewe-cc hook user-prompt-submit < input.json
gewe-cc hook stop < input.json
```

## ⚙️ 配置

配置文件位置：`~/.gewe-cc/config.toml`

```toml
[remote]
# 全局远程模式开关
enabled = false

[notification]
# 通知渠道
channel = "wechat"

# 微信配置
wxid = "wxid_xxxxxxxx"
listen = "0.0.0.0:4399"

[gewe_cli]
# gewe-cli 命令路径
command = "gewe-cli"

# 超时设置（秒，0 表示无限等待）
timeout = 0
```

## 🏗️ 架构

```
┌─────────────────────────────────────┐
│          gewe-cc CLI                 │
├─────────────────────────────────────┤
│  • init    - 初始化环境              │
│  • on/off  - 控制远程模式            │
│  • status  - 查看状态                │
│  • hook    - Hook 处理 (内部)       │
└─────────────────────────────────────┘
         ↓                    ↓
    ┌────▼────┐          ┌────▼────┐
    │gewe-cli │          │ Claude  │
    │         │          │  Code   │
    │微信通信 │          │ Plugin  │
    └─────────┘          └─────────┘
```

## 📚 文档

- [架构设计](./docs/ARCHITECTURE.md)
- [init 命令设计](./docs/INIT_COMMAND.md)
- [会话管理](./docs/SESSION_MANAGEMENT.md)
- [Hook 处理](./docs/HOOK_DESIGN.md)

## 🔧 开发

### 编译

```bash
cargo build
```

### 测试

```bash
cargo test
```

### 发布

使用 cargo-release 管理版本和发布：

```bash
# 安装工具（首次）
cargo install cargo-release git-cliff

# 发布 patch 版本（0.1.0 -> 0.1.1）
cargo release patch

# 发布 minor 版本（0.1.0 -> 0.2.0）
cargo release minor

# 发布 major 版本（0.1.0 -> 1.0.0）
cargo release major

# 预览（dry-run）
cargo release patch --dry-run
```

cargo-release 会自动：
- 使用 git-cliff 生成 CHANGELOG.md
- 更新版本号
- 创建 tag 并推送
- 发布到 crates.io
- 触发 cargo-dist 构建跨平台二进制

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 License

MIT License - 详见 [LICENSE](./LICENSE) 文件

## 🙏 致谢

- [gewe-cli](https://github.com/wangnov/gewe-cli) - 微信消息收发工具
- [Claude Code](https://claude.ai/code) - Anthropic 官方 CLI
- [cargo-dist](https://github.com/axodotdev/cargo-dist) - Rust 二进制分发工具
- [cargo-release](https://github.com/crate-ci/cargo-release) - 版本管理和发布工具
- [git-cliff](https://github.com/orhun/git-cliff) - CHANGELOG 生成器

---

**注意**：此工具仅用于个人学习和合法用途。请遵守微信使用规范。
