# ⚠️ Kiro IDE 导入功能说明

## 问题原因

您遇到的错误：
```
TypeError: Cannot read properties of undefined (reading 'invoke')
```

这是因为 **Kiro IDE 导入**功能需要完整的 OAuth 登录流程，不能简单地调用 `add_account_by_social`。

## 为什么不能直接调用？

参考 `kiro-account-manager` 项目的真实实现，Kiro IDE 导入需要以下步骤：

### 完整的 OAuth 流程

```
1. 用户点击"从 Kiro IDE 导入"
   ↓
2. 调用 kiro_login 命令（启动 OAuth 登录）
   ↓
3. 打开浏览器，用户登录 AWS/Kiro
   ↓
4. 登录成功后，回调返回 authorization code
   ↓
5. 使用 code 换取 access_token 和 refresh_token
   ↓
6. 调用 add_account_by_social 导入账号
   ↓
7. 获取 usage 数据并更新账号状态
```

### 需要的 Tauri 命令

| 命令 | 作用 |
|------|------|
| `kiro_login` | 启动 OAuth 登录流程 |
| `get_kiro_local_token` | 获取本地已登录的 token |
| `add_account_by_social` | 导入 Social 账号 |
| `add_account_by_idc` | 导入 IdC 账号 |

## 当前解决方案

### ✅ 已实现：JSON 导入

您可以使用 JSON 导入功能，这是**最可靠的方式**：

#### 1. 从 kiro-account-manager 导出账号

如果您有 kiro-account-manager 项目：
1. 打开 kiro-account-manager
2. 选择要导出的账号
3. 点击"导出"按钮
4. 保存 JSON 文件

#### 2. 在 kiroOne 中导入

1. 点击"导入账号"按钮
2. 选择"JSON 导入"标签
3. 点击"从文件导入"按钮
4. 选择之前导出的 JSON 文件
5. 点击"导入"按钮

### 📝 JSON 格式示例

**Social 账号（Google/GitHub）：**
```json
[
  {
    "refreshToken": "aor_xxxxxxxx...",
    "provider": "Google"
  },
  {
    "refreshToken": "aor_xxxxxxxx...",
    "provider": "Github",
    "email": "user@github.com"
  }
]
```

**IdC 账号（BuilderId/Enterprise）：**
```json
[
  {
    "refreshToken": "aor_xxxxxxxx...",
    "provider": "BuilderId",
    "clientId": "xxxx",
    "clientSecret": "xxxx",
    "region": "us-east-1"
  },
  {
    "refreshToken": "aor_xxxxxxxx...",
    "provider": "Enterprise",
    "clientId": "xxxx",
    "clientSecret": "xxxx",
    "startUrl": "https://example.awsapps.com/start"
  }
]
```

## 如何完整实现 Kiro IDE 导入

如果您想要完整实现 Kiro IDE 导入功能，需要：

### 1. 实现 OAuth 登录

```rust
#[tauri::command]
pub async fn kiro_login(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    provider: String,
) -> Result<String, String> {
    // 1. 生成 code_verifier 和 state
    // 2. 打开浏览器进行 OAuth 登录
    // 3. 监听回调 URL
    // 4. 获取 authorization code
    // 5. 换取 access_token 和 refresh_token
    // 6. 调用 add_account_by_social
}
```

### 2. 实现本地 Token 获取

```rust
#[tauri::command]
pub async fn get_kiro_local_token() -> Option<KiroToken> {
    // 读取 Kiro IDE 的本地数据库
    // 路径：~/.kiro/auth.db 或类似位置
    // 返回已登录的 token 数据
}
```

### 3. 前端调用流程

```typescript
// 1. 检测本地是否有 Kiro IDE
const hasKiro = await invoke("check_kiro_installed");

if (hasKiro) {
  // 2. 获取本地 token
  const token = await invoke("get_kiro_local_token");
  
  if (token) {
    // 3. 直接导入
    await invoke("add_account_by_social", {
      refreshToken: token.refreshToken,
      provider: token.provider,
      accessToken: token.accessToken
    });
  } else {
    // 4. 需要重新登录
    await invoke("kiro_login", { provider: "Google" });
  }
}
```

## 推荐方案

### 🎯 当前最佳实践

1. **使用 JSON 导入**（已实现，稳定可靠）
   - 从 kiro-account-manager 导出
   - 或者手动准备 JSON 数据
   
2. **暂不使用 Kiro IDE 导入**（需要复杂实现）

### 📊 功能对比

| 导入方式 | 实现难度 | 当前状态 | 推荐度 |
|---------|---------|---------|--------|
| JSON 导入 | ⭐ 简单 | ✅ 已实现 | ⭐⭐⭐⭐⭐ |
| 文件导入 | ⭐⭐ 中等 | ✅ 已实现 | ⭐⭐⭐⭐⭐ |
| Kiro IDE 导入 | ⭐⭐⭐⭐⭐ 复杂 | ❌ 未实现 | - |
| Kiro CLI 导入 | ⭐⭐⭐⭐ 较复杂 | ❌ 未实现 | - |

## 获取 Refresh Token 的方法

### 方法 1：从 kiro-account-manager 导出（推荐）

最简单的方式，直接导出 JSON 文件。

### 方法 2：从 Kiro IDE 数据库提取

```bash
# macOS/Linux
sqlite3 ~/.kiro/auth.db "SELECT * FROM tokens;"

# Windows
sqlite3 %APPDATA%/.kiro/auth.db "SELECT * FROM tokens;"
```

### 方法 3：使用 Kiro CLI

```bash
kiro auth status
# 输出中包含 token 信息
```

## 常见问题

### Q1: 为什么 refreshToken 要以 "aor" 开头？

A: 这是 AWS Cognito 的 refreshToken 格式规范，所有通过 OAuth 获取的 token 都以此开头。

### Q2: 我可以手动获取 token 吗？

A: 可以，但需要：
1. 使用浏览器开发者工具
2. 打开 Kiro IDE 登录页面
3. 在 Network 标签中找到 token 请求
4. 复制 refreshToken 字段

### Q3: IdC 账号和 Social 账号有什么区别？

A:
- **Social 账号**：Google/GitHub 登录，只需要 refreshToken
- **IdC 账号**：BuilderId/Enterprise 登录，需要 clientId + clientSecret

## 总结

✅ **当前可用：**
- JSON 批量导入
- 从文件导入
- 导出账号

❌ **暂未实现：**
- Kiro IDE OAuth 登录
- Kiro CLI 数据库导入

💡 **推荐方案：**
使用 JSON 导入功能，这是最稳定可靠的方式。

---

**更新时间**: 2026年4月6日  
**状态**: JSON 导入功能已完整实现 ✅
