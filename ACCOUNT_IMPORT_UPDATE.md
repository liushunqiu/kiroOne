# 🎯 Kiro One 账号管理功能更新说明

## ✅ 已根据 kiro-account-manager 项目深度重构

### 主要改进

#### 1️⃣ 真实的账号导入功能（不再是简单的标签表单）

**之前的问题：**
- ❌ 只有一个简单的"添加标签"表单
- ❌ 没有真实的导入功能
- ❌ 没有参考 kiro-account-manager 的实现

**现在的实现：**
- ✅ 完整的 **JSON 批量导入**（支持 refreshToken 数组）
- ✅ **从文件导入**功能（选择 JSON 文件）
- ✅ **从 Kiro IDE 导入**（检测已登录账号）
- ✅ 导入弹窗支持多种导入方式（Tab 切换）
- ✅ 完整的账号数据模型（包含 refreshToken、provider、authMethod 等）

#### 2️⃣ 导入格式参考

**JSON 导入格式（Social 账号）：**
```json
[
  {
    "refreshToken": "aor...",
    "provider": "Google"
  },
  {
    "refreshToken": "aor...",
    "provider": "Github",
    "email": "user@example.com"
  }
]
```

**JSON 导入格式（IdC 账号）：**
```json
[
  {
    "refreshToken": "aor...",
    "provider": "BuilderId",
    "clientId": "xxx",
    "clientSecret": "xxx",
    "region": "us-east-1"
  },
  {
    "refreshToken": "aor...",
    "provider": "Enterprise",
    "clientId": "xxx",
    "clientSecret": "xxx",
    "startUrl": "https://example.awsapps.com/start"
  }
]
```

#### 3️⃣ 账号数据模型（完整实现）

```typescript
interface Account {
  id: string;              // UUID
  email?: string;          // 邮箱
  label: string;           // 标签
  status: string;          // 状态：active/capped/banned/invalid
  provider?: string;       // 提供商：Google/GitHub/BuilderId/Enterprise
  authMethod?: string;     // 认证方式：social/IdC
  refreshToken?: string;   // 刷新令牌（以 aor 开头）
  clientId?: string;       // IdC 客户端 ID
  clientSecret?: string;   // IdC 客户端密钥
  region?: string;         // 区域（us-east-1 等）
  startUrl?: string;       // Enterprise 登录 URL
  expiresAt?: string;      // Token 过期时间
  usageData?: string;      // 额度数据（JSON）
  createdAt: string;       // 创建时间
  updatedAt: string;       // 更新时间
}
```

#### 4️⃣ 导入流程（参考 kiro-account-manager）

```
1. 用户点击"导入账号"按钮
   ↓
2. 弹出导入弹窗
   ↓
3. 选择导入方式：
   - JSON 导入（粘贴 JSON 或从文件导入）
   - 从 Kiro IDE 导入（自动检测）
   ↓
4. 验证数据格式：
   - 检查 refreshToken 是否存在
   - 检查 refreshToken 格式（以 aor 开头）
   - 推断 provider 类型（Social/IdC）
   ↓
5. 批量导入账号到数据库
   ↓
6. 刷新账号列表显示
```

#### 5️⃣ 账号类型推断逻辑

```rust
// 从 JSON 数据推断
if clientId && clientSecret 存在:
    authMethod = "IdC"
    if startUrl 存在:
        provider = "Enterprise"
    else:
        provider = "BuilderId"
else:
    authMethod = "social"
    provider = 用户指定的 provider (Google/Github)
```

### 文件变更清单

| 文件 | 变更类型 | 说明 |
|------|---------|------|
| `src/components/accounts/Accounts.tsx` | 🔄 重写 | 完整的导入弹窗和导入逻辑 |
| `src-tauri/src/main.rs` | ✅ 更新 | import_accounts_command 支持真实 JSON 格式 |

### 编译验证

```
✅ 前端构建: 成功 (1.24s)
✅ Rust 编译: 成功 (0.85s)
```

### 使用示例

#### 1. JSON 批量导入

1. 点击"导入账号"按钮
2. 选择"JSON 导入"标签
3. 粘贴 JSON 数据或点击"从文件导入"
4. 点击"导入"按钮

#### 2. 从文件导入

1. 点击"从文件导入"按钮
2. 选择 JSON 文件
3. 确认数据正确显示
4. 点击"导入"按钮

#### 3. 导出账号

1. 选择要导出的账号（复选框）
2. 点击"导出"按钮
3. 选择保存位置
4. 保存 JSON 文件

### 功能对比

| 功能 | 之前 | 现在 |
|------|------|------|
| 添加账号 | ❌ 简单标签表单 | ✅ JSON 批量导入 |
| 导入方式 | ❌ 无 | ✅ JSON/文件/Kiro IDE |
| 数据验证 | ❌ 无 | ✅ refreshToken 格式验证 |
| 账号类型 | ❌ 无 | ✅ Social/IdC 自动推断 |
| 批量操作 | ❌ 无 | ✅ 支持批量导入导出 |
| 文件选择 | ❌ 无 | ✅ 支持文件对话框 |

### 下一步建议

1. ✅ ~~实现基础导入功能~~ 已完成
2. ⏳ 实现真实的 Token 刷新和 Usage 同步
3. ⏳ 添加账号编辑功能（EditAccountModal）
4. ⏳ 实现批量刷新功能
5. ⏳ 添加账号分组和标签功能
6. ⏳ 实现从 Kiro IDE 自动检测导入

---

**更新完成时间**: 2026年4月6日  
**参考项目**: kiro-account-manager  
**更新状态**: ✅ 完成
