# Kiro One 未实现功能清单

## 🔴 关键缺失功能 (影响核心功能)

### 1. API Key 认证 ❌
**位置**: `src-tauri/src/gateway/server.rs`

**问题**: 网关没有验证客户端的 API Key
- 当前任何人都可以访问 `http://localhost:8710` 的所有端点
- `GatewayConfig` 有 `api_key` 字段,但从未使用

**参考**: KiroaaS 的 `routes_openai.py` 和 `routes_anthropic.py` 都有 `verify_api_key` 中间件

**影响**: 安全风险,任何本地进程都可以使用你的 Kiro 账号

---

### 2. 代理设置 ❌
**位置**: `src/components/gateway/Gateway.tsx` (UI 存在) vs `src-tauri/src/gateway/server.rs` (后端缺失)

**问题**: 
- UI 有 `proxyEnabled` 和 `proxyUrl` 字段
- `GatewayConfig` 结构体没有这些字段
- 后端不支持通过代理访问 Kiro API

**参考**: KiroaaS 的 `main.py` 设置 `HTTP_PROXY` 环境变量

**影响**: 无法在需要代理的网络环境中使用

---

### 3. Claude Code 配置同步 ❌
**位置**: 完全缺失

**问题**: README 声称支持 "Claude Code 配置自动同步",但代码中完全没有实现
- 没有读取 `~/.claude/settings.json`
- 没有写入配置文件
- 切换供应商不会影响 Claude Code

**参考**: cc-switch 的 `session_manager/providers/claude.rs`

**影响**: 无法与 Claude Code CLI 集成,这是 README 承诺的核心功能之一

---

## 🟡 次要缺失功能 (影响用户体验)

### 4. 仪表盘快速操作 ⚠️
**位置**: `src/components/dashboard/Dashboard.tsx` (第 128-136 行)

**问题**: 三个按钮没有事件处理器
```tsx
<button className="...">添加账号</button>  // 无 onClick
<button className="...">添加供应商</button> // 无 onClick
<button className="...">启动网关</button>   // 无 onClick
```

**影响**: 按钮无法点击,用户体验差

---

### 5. 活动日志 ❌
**位置**: `src/components/dashboard/Dashboard.tsx` (第 140-149 行)

**问题**: 显示 "暂无活动记录",但没有任何记录活动的逻辑
- 没有日志存储
- 没有操作记录

**参考**: kiro-account-manager 可能有操作日志

**影响**: 无法追踪历史操作

---

### 6. 错误重试逻辑 ❌
**位置**: `src-tauri/src/gateway/server.rs` 和 `src-tauri/src/api_client.rs`

**问题**: 
- API 调用失败直接返回错误
- 没有 403/429/5xx 重试
- 没有指数退避

**参考**: KiroaaS 的 `http_client.py` 有完整的重试逻辑

**影响**: 网络抖动或 API 限流时用户体验差

---

### 7. 流式响应结束标记 ⚠️
**位置**: `src-tauri/src/gateway/server.rs`

**问题**: 流式响应没有发送结束事件
- OpenAI 格式缺少 `data: [DONE]`
- Anthropic 格式缺少 `message_stop` 事件

**影响**: 某些客户端可能不知道流何时结束

---

## 🟢 数据结构不匹配 (前后端不一致)

### 8. GatewayConfig 字段缺失 ⚠️
**前端** (`Gateway.tsx`):
```typescript
interface GatewayConfig {
  port: number;
  apiKey: string;
  isRunning: boolean;
  defaultModel?: string;
  proxyEnabled: boolean;    // ❌ 后端没有
  proxyUrl?: string;         // ❌ 后端没有
  createdAt: string;         // ❌ 后端没有
  updatedAt: string;         // ❌ 后端没有
}
```

**后端** (`state.rs`):
```rust
pub struct GatewayConfig {
    pub port: u16,
    pub api_key: String,
    pub is_running: bool,
    pub default_model: Option<String>,
    // 缺少 proxy_enabled, proxy_url, created_at, updated_at
}
```

**影响**: 前端显示的字段后端无法保存

---

## 🔵 优化建议 (非必需但有价值)

### 9. AWS EventStream 完整解析 ⚠️
**位置**: `src-tauri/src/gateway/streaming.rs`

**问题**: 当前是简化实现,假设 JSON 行而非二进制协议

**建议**: 使用 `aws-smithy-eventstream` crate

---

### 10. Token 自动刷新 ⚠️
**位置**: `src-tauri/src/api_client.rs`

**问题**: 
- Token 过期前不会自动刷新
- 需要手动点击"同步"按钮

**参考**: kiro-account-manager 有自动刷新机制

**建议**: 添加后台定时任务

---

### 11. 账号轮询/负载均衡 ❌
**位置**: `src-tauri/src/gateway/server.rs`

**问题**: 
- 总是使用第一个 active 账号
- 没有轮询或负载均衡
- 账号额度用尽不会自动切换

**参考**: kiro-account-manager 有自动换号功能

---

### 12. 系统信息错误 ⚠️
**位置**: `src/components/dashboard/Dashboard.tsx` (第 163 行)

**问题**: 显示 "数据库: SQLite",但实际使用 JSON 文件存储

---

## 📊 功能完成度统计

| 模块 | 声称完成 | 实际完成 | 缺失功能 |
|------|---------|---------|---------|
| 账号管理 | ✅ | ✅ 95% | Token 自动刷新 |
| API 网关 | ✅ | ⚠️ 70% | API Key 认证、代理、重试 |
| 供应商管理 | ✅ | ⚠️ 60% | Claude Code 同步 |
| 数据持久化 | ✅ | ✅ 100% | - |
| 流式响应 | ✅ | ⚠️ 80% | 结束标记、完整解析 |
| 仪表盘 | ✅ | ⚠️ 50% | 快速操作、活动日志 |

---

## 🎯 优先级建议

### 立即修复 (P0 - 阻塞性问题)
1. **API Key 认证** - 安全问题
2. **GatewayConfig 字段同步** - 前后端不一致
3. **仪表盘快速操作** - 按钮无法使用

### 高优先级 (P1 - 核心功能)
4. **代理设置** - 网络环境限制
5. **Claude Code 配置同步** - README 承诺的功能
6. **错误重试逻辑** - 稳定性

### 中优先级 (P2 - 用户体验)
7. **流式响应结束标记**
8. **活动日志**
9. **Token 自动刷新**

### 低优先级 (P3 - 优化)
10. **AWS EventStream 完整解析**
11. **账号轮询/负载均衡**
12. **系统信息修正**

---

## 📝 总结

**实际完成度**: 约 **75-80%** (而非之前估计的 95%)

**关键问题**:
- ❌ API Key 认证缺失 (安全风险)
- ❌ Claude Code 同步完全未实现 (README 虚假宣传)
- ❌ 代理设置未实现 (功能缺失)
- ⚠️ 前后端数据结构不匹配

**建议**: 优先实现 P0 和 P1 功能,才能称为"生产可用"。
