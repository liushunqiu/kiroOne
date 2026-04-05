# AI Unified Manager

> Kiro 账号管理与 API 网关统一工具 - 纯 Rust 实现

一个跨平台桌面应用，集成 Kiro 账号管理和 API 网关功能，为 Claude Code 提供无缝的 API 服务。

## ✨ 特性

- 🎯 **纯 Rust 实现** - 无 Python 依赖，性能更好，体积更小
- 🖥️ **跨平台支持** - Windows 10+/macOS 12+/Linux
- 🔑 **账号管理** - 多账号导入、切换、配额监控
- 🌐 **API 网关** - 兼容 OpenAI 和 Anthropic 协议
- 🤖 **Claude Code 集成** - 一键配置，开箱即用
- 📊 **实时监控** - 配额进度、Token 过期提醒
- 🎨 **现代 UI** - 支持亮色/暗色主题

## 🚀 快速开始

### 前置要求

- Node.js 18+
- Rust 1.70+
- (Windows) Build Tools for Windows
- (macOS) Xcode Command Line Tools

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri:dev
```

### 构建生产版本

```bash
npm run tauri:build
```

## 📖 使用说明

### 1. 导入账号

1. 打开应用，进入「账号管理」
2. 点击「导入账号」
3. 选择导入方式 (JSON/IDE/CLI)
4. 完成导入

### 2. 启动 API 网关

网关默认在应用启动时自动运行，监听 `http://127.0.0.1:8000`

### 3. 配置 Claude Code

#### 方法一：一键配置

在应用中点击「一键配置 Claude Code」按钮

#### 方法二：手动配置

在 Claude Code 中设置环境变量:

```bash
export ANTHROPIC_BASE_URL=http://127.0.0.1:8000
export ANTHROPIC_API_KEY=sk-local-kiro-gateway
```

### 4. API 端点

| 端点 | 协议 | 说明 |
|------|------|------|
| `/v1/chat/completions` | OpenAI | 聊天补全 |
| `/v1/messages` | Anthropic | 消息补全 |
| `/v1/models` | - | 获取模型列表 |

## 🏗️ 技术架构

```
┌─────────────────────────────────────┐
│          前端 UI 层                  │
│  React + TypeScript + TailwindCSS   │
└─────────────────────────────────────┘
            ↕ Tauri IPC
┌─────────────────────────────────────┐
│      Tauri 2 Rust 后端               │
│                                      │
│  ┌──────────────────────────────┐   │
│  │ Axum HTTP 服务器             │   │
│  │ - OpenAI 兼容路由            │   │
│  │ - Anthropic 兼容路由         │   │
│  └──────────────────────────────┘   │
│                                      │
│  ┌──────────────────────────────┐   │
│  │ Kiro API 客户端              │   │
│  └──────────────────────────────┘   │
│                                      │
│  ┌──────────────────────────────┐   │
│  │ SQLite 数据库                │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
```

## 📁 项目结构

```
ai-unified-manager/
├── src/                          # 前端 React
│   ├── components/
│   │   ├── accounts/             # 账号管理
│   │   ├── dashboard/            # 仪表盘
│   │   ├── gateway/              # API 网关
│   │   └── settings/             # 设置
│   └── ...
│
├── src-tauri/                    # Tauri Rust 后端
│   ├── src/
│   │   ├── commands/             # Tauri Commands
│   │   ├── services/             # 业务逻辑
│   │   ├── models/               # 数据模型
│   │   └── db/                   # 数据库
│   └── ...
│
└── package.json
```

## 🔧 开发指南

### 添加新功能

1. 在 `src-tauri/src/services/` 中实现业务逻辑
2. 在 `src-tauri/src/commands/` 中创建 Tauri Command
3. 在 `src/commands/` 中注册 Command
4. 在前端调用 Command

### 数据库操作

数据库文件位于:
- Windows: `%APPDATA%/ai-unified-manager/ai-unified-manager.db`
- macOS: `~/Library/Application Support/ai-unified-manager/ai-unified-manager.db`
- Linux: `~/.config/ai-unified-manager/ai-unified-manager.db`

## 📝 API 示例

### OpenAI 兼容

```bash
curl http://127.0.0.1:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "kiro-default",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

### Anthropic 兼容

```bash
curl http://127.0.0.1:8000/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: sk-local-kiro-gateway" \
  -d '{
    "model": "kiro-default",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ],
    "max_tokens": 1024
  }'
```

## ⚠️ 注意事项

- 本工具仅供个人使用
- 请勿用于商业用途
- 遵守 Kiro 服务条款

## 📄 许可证

MIT License

## 🙏 致谢

本项目参考了以下开源项目:

- [kiro-account-manager](https://github.com/hj01857655/kiro-account-manager)
- [KiroaaS](https://github.com/hnewcity/KiroaaS)
- [cc-switch](https://github.com/farion1231/cc-switch)
