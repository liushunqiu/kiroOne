# Kiro One - AI 账号管理与 API 网关统一平台

> 整合了 kiro-account-manager、KiroaaS 和 cc-switch 三大项目的核心功能

## 📋 项目简介

**Kiro One** 是一个全功能的 AI 开发工具统一管理平台，提供：

1. **账号管理** - 多账号 CRUD、导入导出、额度监控、状态同步
2. **API 网关** - OpenAI/Anthropic 兼容 API、流式响应、额度查询
3. **供应商切换** - 一键切换 AI 供应商、CLI 配置自动同步

## ✨ 核心功能

### 1️⃣ 账号管理（参考 kiro-account-manager）

- ✅ 账号 CRUD（添加、编辑、删除、查询）
- ✅ 批量导入/导出（JSON 格式）
- ✅ 额度监控（Usage 实时查询）
- ✅ 状态同步（Token 自动刷新）
- ✅ 多 Provider 支持（Google/GitHub/Enterprise）
- ✅ 状态管理（active/capped/banned/invalid）

### 2️⃣ API 网关（参考 KiroaaS）

- ✅ OpenAI 兼容 API (`/v1/chat/completions`)
- ✅ Anthropic 兼容 API (`/v1/messages`)
- ✅ 额度查询 (`/usage`, `/account`)
- ✅ 模型列表 (`/v1/models`)
- ✅ 健康检查 (`/health`)
- ✅ API Key 认证

### 3️⃣ 供应商管理（参考 cc-switch）

- ✅ 供应商 CRUD
- ✅ 一键切换供应商
- ✅ Claude Code 配置自动同步
- ✅ 预设供应商模板
- ✅ 多 API 格式支持（Anthropic/OpenAI Chat/OpenAI Responses）

## 🏗️ 技术架构

### 前端技术栈

```
React 18 + TypeScript
├── Vite (构建工具)
├── React Router (路由)
├── TanStack Query (数据请求)
├── TailwindCSS (样式)
└── Lucide React (图标)
```

### 后端技术栈

```
Tauri 2.0 (Rust)
├── SQLite (数据库)
├── Axum (HTTP 服务器)
├── Reqwest (HTTP 客户端)
└── Tokio (异步运行时)
```

## 📁 项目结构

```
kiroOne/
├── src/                          # 前端 React 应用
│   ├── components/
│   │   ├── accounts/            # 账号管理组件
│   │   │   └── Accounts.tsx     # 账号列表、CRUD、导入导出
│   │   ├── gateway/             # API 网关组件
│   │   │   └── Gateway.tsx      # 网关配置、服务控制
│   │   ├── settings/            # 供应商设置组件
│   │   │   └── Settings.tsx     # 供应商管理、切换
│   │   ├── dashboard/           # 仪表盘组件
│   │   │   └── Dashboard.tsx    # 统计、快速操作
│   │   ├── layout/              # 布局组件
│   │   │   └── Sidebar.tsx      # 侧边栏导航
│   │   └── ui/                  # UI 基础组件
│   │       └── Card.tsx         # 卡片组件
│   ├── App.tsx                  # 主应用路由
│   └── main.tsx                 # 前端入口
│
├── src-tauri/                   # 后端 Rust 应用
│   ├── src/
│   │   └── main.rs              # Tauri 主入口 + 所有命令
│   ├── Cargo.toml               # Rust 依赖
│   └── tauri.conf.json          # Tauri 配置
│
└── package.json                 # Node.js 依赖
```

## 🚀 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
# 启动前端 + Tauri 桌面应用
npm run tauri:dev

# 仅启动前端
npm run dev
```

### 构建生产版本

```bash
# 构建前端
npm run build

# 构建桌面应用
npm run tauri:build
```

## 📖 使用指南

### 1. 添加账号

1. 进入"账号管理"页面
2. 点击"添加账号"按钮
3. 输入标签和邮箱
4. 点击"同步"按钮刷新 Token 和额度

### 2. 导入/导出账号

**导出：**
1. 选择要导出的账号（复选框）
2. 点击"导出"按钮
3. 选择保存位置

**导入：**
1. 点击"导入"按钮
2. 选择之前导出的 JSON 文件
3. 等待导入完成

### 3. 配置 API 网关

1. 进入"API 网关"页面
2. 配置端口（默认 8710）
3. 生成或设置 API Key
4. 点击"启动服务"

**使用示例：**

```bash
# 查询模型列表
curl http://localhost:8710/v1/models \
  -H "Authorization: Bearer YOUR_API_KEY"

# 发送聊天请求
curl http://localhost:8710/v1/chat/completions \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'

# 查询额度
curl http://localhost:8710/usage \
  -H "Authorization: Bearer YOUR_API_KEY"
```

### 4. 管理供应商

**添加供应商：**
1. 进入"供应商设置"页面
2. 点击"添加供应商"
3. 填写名称、API 地址、密钥和格式
4. 或从预设中快速添加

**切换供应商：**
1. 在供应商列表中点击闪电图标
2. 确认切换
3. Claude Code 配置会自动更新

## 🔌 API 文档

### 认证

所有 API 请求需要在 Header 中添加 API Key：

```
Authorization: Bearer YOUR_API_KEY
```

### 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 健康检查 |
| GET | `/v1/models` | 获取模型列表 |
| POST | `/v1/chat/completions` | OpenAI 兼容聊天 |
| POST | `/v1/messages` | Anthropic 兼容消息 |
| GET | `/usage` | 查询额度使用 |
| GET | `/account` | 查询账号信息 |

### 请求示例

**OpenAI 格式：**

```json
{
  "model": "claude-sonnet-4",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "What is Rust?"}
  ],
  "stream": false,
  "temperature": 0.7
}
```

**Anthropic 格式：**

```json
{
  "model": "claude-sonnet-4",
  "max_tokens": 1024,
  "messages": [
    {"role": "user", "content": "What is Rust?"}
  ],
  "system": "You are a helpful assistant."
}
```

## 📊 数据模型

### Account（账号）

```typescript
interface Account {
  id: string;              // UUID
  email?: string;          // 邮箱
  label: string;           // 标签
  status: string;          // 状态：active/capped/banned/invalid
  provider?: string;       // 提供商：Google/GitHub/Enterprise
  authMethod?: string;     // 认证方式：social/IdC
  accessToken?: string;    // 访问令牌
  refreshToken?: string;   // 刷新令牌
  expiresAt?: string;      // 过期时间
  userId?: string;         // 用户 ID
  usageData?: string;      // 额度数据（JSON）
  createdAt: string;       // 创建时间
  updatedAt: string;       // 更新时间
}
```

### Provider（供应商）

```typescript
interface Provider {
  id: string;              // UUID
  name: string;            // 名称
  apiBaseUrl: string;      // API 基础 URL
  apiKey: string;          // API 密钥
  apiFormat: string;       // API 格式
  isActive: boolean;       // 是否活跃
  createdAt: string;       // 创建时间
  updatedAt: string;       // 更新时间
}
```

### GatewayConfig（网关配置）

```typescript
interface GatewayConfig {
  port: number;            // 端口号
  apiKey: string;          // API 密钥
  isRunning: boolean;      // 是否运行中
  defaultModel?: string;   // 默认模型
}
```

## 🛠️ 开发指南

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/main.rs` 中添加函数：

```rust
#[tauri::command]
fn my_new_command(state: tauri::State<AppState>) -> Result<String, String> {
    // 实现逻辑
    Ok("success".to_string())
}
```

2. 在 `main()` 函数中注册：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 其他命令
    my_new_command,
])
```

### 添加新的前端页面

1. 在 `src/components/` 创建组件
2. 在 `src/App.tsx` 添加路由
3. 在 `src/components/layout/Sidebar.tsx` 添加导航

## 🔍 常见问题

### Q: 编译失败怎么办？

A: 确保已安装 Rust 和 Node.js 依赖：
```bash
rustc --version
node --version
npm --version
```

### Q: 如何重置数据？

A: 删除数据文件（默认位置）：
- Windows: `%APPDATA%/.kiro-one/`
- macOS: `~/Library/Application Support/.kiro-one/`
- Linux: `~/.local/share/.kiro-one/`

### Q: API 网关无法启动？

A: 检查端口是否被占用，尝试更换端口号。

## 📝 更新日志

### v0.1.0 (2026-04-06)

- ✅ 初始版本发布
- ✅ 账号管理功能
- ✅ API 网关基础功能
- ✅ 供应商管理和切换
- ✅ 完整前端 UI

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

## 🙏 致谢

本项目参考了以下优秀项目：

- [kiro-account-manager](https://github.com/...) - 账号管理
- [KiroaaS](https://github.com/...) - API 网关
- [cc-switch](https://github.com/...) - 供应商切换

---

**Made with ❤️ using Tauri + React**
