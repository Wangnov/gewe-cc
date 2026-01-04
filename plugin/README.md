# gewe-cc-plugin

Claude Code 远程协作模式插件。

## 功能

- ✅ **远程模式控制**：通过 `>remote-on/off/status` 命令控制远程模式
- ✅ **自动等待微信指令**：任务完成后自动发送微信通知并等待回复
- ✅ **循环工作流**：根据微信回复继续工作或停止
- ✅ **零 Python 依赖**：完全依赖 gewe-cc 二进制工具

## 依赖

- [gewe-cc](https://github.com/wangnov/gewe-cc) - 远程控制命令行工具
- [gewe-cli](https://github.com/wangnov/gewe-cli) - 微信消息收发工具
- [Claude Code](https://claude.ai/code) - Anthropic 官方 CLI

## 安装

### 方式 1：Claude Code 官方命令（推荐）

```bash
claude plugin install gewe-cc
```

### 方式 2：从 GitHub 安装

```bash
claude plugin install https://github.com/wangnov/gewe-cc
```

### 方式 3：本地开发模式

```bash
# 克隆仓库
git clone https://github.com/wangnov/gewe-cc.git
cd gewe-cc

# 安装 plugin
claude plugin install --local ./plugin
```

## 使用方法

### 1. 初始化 gewe-cc

```bash
# 安装 gewe-cc
cargo install gewe-cc

# 初始化配置
gewe-cc init
```

### 2. 启用远程模式

在 Claude Code 中输入：

```
>remote-on
```

### 3. 正常工作

```
创建一个文件 test.txt
```

### 4. 任务完成后自动

- 发送微信通知
- 等待你的回复
- 根据回复继续工作或停止

### 5. 禁用远程模式

在 Claude Code 中输入：

```
>remote-off
```

或通过微信回复：`停止`

## 文件结构

```
plugin/
├── .claude-plugin/
│   └── plugin.json          # Plugin 元数据
├── hooks/
│   └── hooks.json           # Hook 配置
└── skills/
    └── remote-control/
        └── SKILL.md         # 远程控制逻辑
```

## 工作原理

```
1. >remote-on
     ↓
   启用全局远程模式

2. 完成任务
     ↓
   Stop Hook 检测到远程模式
     ↓
   激活 remote-control Skill

3. Skill 执行
     ↓
   发送微信通知 (gewe-cli)
     ↓
   等待回复
     ↓
   根据回复继续或停止
```

## License

MIT
