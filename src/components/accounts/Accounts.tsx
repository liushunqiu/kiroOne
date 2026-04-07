import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/Card";
import { RefreshCw, Download, Upload, Trash2, Edit, FileJson, Database } from "lucide-react";

interface Account {
  id: string;
  email?: string;
  label: string;
  status: string;
  provider?: string;
  authMethod?: string;
  refreshToken?: string;
  clientId?: string;
  clientSecret?: string;
  region?: string;
  startUrl?: string;
  expiresAt?: string;
  usageData?: string;
  createdAt: string;
  updatedAt: string;
}

export function Accounts() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [syncingId, setSyncingId] = useState<string | null>(null);
  const [showImportModal, setShowImportModal] = useState(false);
  const [importTab, setImportTab] = useState<"json" | "kiro" | "kiro-cli">("json");
  const [jsonText, setJsonText] = useState("");
  const [importing, setImporting] = useState(false);
  const [editingAccount, setEditingAccount] = useState<Account | null>(null);
  const [editForm, setEditForm] = useState({ label: "", email: "", status: "active" });

  // 加载账号列表
  const loadAccounts = async () => {
    try {
      setLoading(true);
      const data = await invoke<Account[]>("get_accounts");
      setAccounts(data);
    } catch (error) {
      console.error("Failed to load accounts:", error);
    } finally {
      setLoading(false);
    }
  };

  // 显示导入弹窗
  const handleShowImport = () => {
    setShowImportModal(true);
    setImportTab("json");
    setJsonText("");
  };

  // JSON 批量导入
  const handleJsonImport = async () => {
    if (!jsonText.trim()) {
      alert("请输入 JSON 数据");
      return;
    }

    try {
      setImporting(true);
      const count = await invoke<number>("import_accounts_command", { json: jsonText });
      alert(`成功导入 ${count} 个账号`);
      setShowImportModal(false);
      setJsonText("");
      await loadAccounts();
    } catch (error) {
      console.error("Failed to import:", error);
      alert("导入失败: " + error);
    } finally {
      setImporting(false);
    }
  };

  // 从文件导入
  const handleFileImport = async () => {
    try {
      const filePath = await open({
        multiple: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });

      if (!filePath) return;

      const content = await readTextFile(filePath);
      setJsonText(content);
    } catch (error) {
      console.error("Failed to read file:", error);
      alert("读取文件失败");
    }
  };

  // 导出账号
  const handleExport = async () => {
    if (selectedIds.size === 0) {
      alert("请先选择要导出的账号");
      return;
    }

    try {
      const json = await invoke<string>("export_accounts_command", {
        ids: Array.from(selectedIds),
      });

      const filePath = await save({
        defaultPath: `kiro-accounts-${Date.now()}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });

      if (filePath) {
        await writeTextFile(filePath, json);
        alert("导出成功");
      }
    } catch (error) {
      console.error("Failed to export:", error);
      alert("导出失败");
    }
  };

  // 同步账号
  const handleSyncAccount = async (id: string) => {
    setSyncingId(id);
    try {
      await invoke("sync_account", { id });
      await loadAccounts();
    } catch (error) {
      console.error("Failed to sync account:", error);
      alert("同步账号失败");
    } finally {
      setSyncingId(null);
    }
  };

  // 删除账号
  const handleDeleteAccount = async (id: string) => {
    if (!confirm("确定要删除这个账号吗?")) return;

    try {
      await invoke("delete_account", { id });
      await loadAccounts();
    } catch (error) {
      console.error("Failed to delete account:", error);
      alert("删除账号失败");
    }
  };

  // 批量删除
  const handleBatchDelete = async () => {
    if (selectedIds.size === 0) return;
    if (!confirm(`确定要删除选中的 ${selectedIds.size} 个账号吗?`)) return;

    try {
      for (const id of selectedIds) {
        await invoke("delete_account", { id });
      }
      setSelectedIds(new Set());
      await loadAccounts();
    } catch (error) {
      console.error("Failed to batch delete:", error);
      alert("批量删除失败");
    }
  };

  // 打开编辑模态框
  const handleEditAccount = (account: Account) => {
    setEditingAccount(account);
    setEditForm({
      label: account.label,
      email: account.email || "",
      status: account.status,
    });
  };

  // 保存编辑
  const handleSaveEdit = async () => {
    if (!editingAccount) return;
    if (!editForm.label.trim()) {
      alert("标签不能为空");
      return;
    }

    try {
      await invoke("update_account", {
        id: editingAccount.id,
        label: editForm.label,
        status: editForm.status,
      });
      setEditingAccount(null);
      await loadAccounts();
    } catch (error) {
      console.error("Failed to update account:", error);
      alert("更新失败: " + error);
    }
  };

  // 切换选择
  const toggleSelect = (id: string) => {
    const newSelected = new Set(selectedIds);
    if (newSelected.has(id)) {
      newSelected.delete(id);
    } else {
      newSelected.add(id);
    }
    setSelectedIds(newSelected);
  };

  // 全选/取消全选
  const toggleSelectAll = () => {
    if (selectedIds.size === accounts.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(accounts.map((a) => a.id)));
    }
  };

  // 获取状态颜色
  const getStatusColor = (status: string) => {
    switch (status) {
      case "active":
        return "bg-green-100 text-green-800";
      case "capped":
        return "bg-yellow-100 text-yellow-800";
      case "banned":
        return "bg-red-100 text-red-800";
      case "invalid":
        return "bg-orange-100 text-orange-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  };

  // 解析 usage data
  const getUsageInfo = (usageData?: string) => {
    if (!usageData) return null;
    try {
      const data = JSON.parse(usageData);
      const breakdown = data.usageBreakdownList?.[0];
      if (!breakdown) return null;

      const current = breakdown.currentUsageWithPrecision || breakdown.currentUsage || 0;
      const limit = breakdown.usageLimitWithPrecision || breakdown.usageLimit || 0;
      const percentage = limit > 0 ? (current / limit) * 100 : 0;

      return { current, limit, percentage };
    } catch {
      return null;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">账号管理</h1>
        <div className="flex gap-2">
          <button
            onClick={handleShowImport}
            className="px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600 flex items-center gap-2"
          >
            <Upload size={16} />
            导入账号
          </button>
          <button
            onClick={handleExport}
            disabled={selectedIds.size === 0}
            className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50 flex items-center gap-2"
          >
            <Download size={16} />
            导出
          </button>
          <button
            onClick={handleBatchDelete}
            disabled={selectedIds.size === 0}
            className="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 disabled:opacity-50 flex items-center gap-2"
          >
            <Trash2 size={16} />
            删除
          </button>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <span>账号列表 ({accounts.length})</span>
            <button
              onClick={loadAccounts}
              className="px-3 py-1 text-sm bg-gray-100 rounded hover:bg-gray-200 flex items-center gap-1"
            >
              <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
              刷新
            </button>
          </CardTitle>
        </CardHeader>
        <CardContent>
          {loading && accounts.length === 0 ? (
            <div className="text-center py-8">加载中...</div>
          ) : accounts.length === 0 ? (
            <div className="text-center py-16">
              <div className="w-24 h-24 mx-auto mb-6 rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg">
                <Database size={48} className="text-white" />
              </div>
              <h3 className="text-xl font-bold text-gray-900 mb-2">还没有账号</h3>
              <p className="text-sm text-gray-500 mb-6">导入账号开始管理你的 Kiro IDE 账户</p>
              <button
                onClick={handleShowImport}
                className="px-6 py-3 rounded-xl text-sm font-medium text-white bg-gradient-to-r from-indigo-500 to-purple-600 shadow-lg hover:shadow-xl transition-all duration-200 hover:scale-105 flex items-center gap-2 mx-auto"
              >
                <Upload size={18} />
                导入账号
              </button>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b">
                    <th className="p-3 text-left">
                      <input
                        type="checkbox"
                        checked={selectedIds.size === accounts.length && accounts.length > 0}
                        onChange={toggleSelectAll}
                      />
                    </th>
                    <th className="p-3 text-left">标签</th>
                    <th className="p-3 text-left">邮箱</th>
                    <th className="p-3 text-left">提供商</th>
                    <th className="p-3 text-left">状态</th>
                    <th className="p-3 text-left">额度使用</th>
                    <th className="p-3 text-left">操作</th>
                  </tr>
                </thead>
                <tbody>
                  {accounts.map((account) => {
                    const usage = getUsageInfo(account.usageData);
                    return (
                      <tr key={account.id} className="border-b hover:bg-gray-50">
                        <td className="p-3">
                          <input
                            type="checkbox"
                            checked={selectedIds.has(account.id)}
                            onChange={() => toggleSelect(account.id)}
                          />
                        </td>
                        <td className="p-3 font-medium">{account.label}</td>
                        <td className="p-3 text-sm text-gray-600">
                          {account.email || "-"}
                        </td>
                        <td className="p-3 text-sm">{account.provider || "-"}</td>
                        <td className="p-3">
                          <span
                            className={`px-2 py-1 rounded text-xs ${getStatusColor(
                              account.status
                            )}`}
                          >
                            {account.status}
                          </span>
                        </td>
                        <td className="p-3">
                          {usage ? (
                            <div className="w-32">
                              <div className="flex justify-between text-xs mb-1">
                                <span>{usage.current.toFixed(0)}</span>
                                <span>{usage.limit.toFixed(0)}</span>
                              </div>
                              <div className="w-full bg-gray-200 rounded-full h-2">
                                <div
                                  className="bg-blue-500 h-2 rounded-full"
                                  style={{ width: `${Math.min(usage.percentage, 100)}%` }}
                                />
                              </div>
                            </div>
                          ) : (
                            <span className="text-xs text-gray-400">未同步</span>
                          )}
                        </td>
                        <td className="p-3">
                          <div className="flex gap-2">
                            <button
                              onClick={() => handleSyncAccount(account.id)}
                              disabled={syncingId === account.id}
                              className="p-1 text-blue-500 hover:bg-blue-50 rounded disabled:opacity-50"
                              title="同步"
                            >
                              <RefreshCw
                                size={16}
                                className={syncingId === account.id ? "animate-spin" : ""}
                              />
                            </button>
                            <button
                              onClick={() => handleEditAccount(account)}
                              className="p-1 text-green-500 hover:bg-green-50 rounded"
                              title="编辑"
                            >
                              <Edit size={16} />
                            </button>
                            <button
                              onClick={() => handleDeleteAccount(account.id)}
                              className="p-1 text-red-500 hover:bg-red-50 rounded"
                              title="删除"
                            >
                              <Trash2 size={16} />
                            </button>
                          </div>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </CardContent>
      </Card>

      {/* 编辑账号模态框 */}
      {editingAccount && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-md">
            <div className="p-6 border-b border-gray-200">
              <h2 className="text-xl font-bold">编辑账号</h2>
            </div>
            <div className="p-6 space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2">标签</label>
                <input
                  type="text"
                  value={editForm.label}
                  onChange={(e) => setEditForm({ ...editForm, label: e.target.value })}
                  className="w-full px-3 py-2 border rounded"
                  placeholder="账号标签"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">邮箱</label>
                <input
                  type="email"
                  value={editForm.email}
                  onChange={(e) => setEditForm({ ...editForm, email: e.target.value })}
                  className="w-full px-3 py-2 border rounded"
                  placeholder="邮箱地址"
                  disabled
                />
                <p className="text-xs text-gray-500 mt-1">邮箱不可修改</p>
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">状态</label>
                <select
                  value={editForm.status}
                  onChange={(e) => setEditForm({ ...editForm, status: e.target.value })}
                  className="w-full px-3 py-2 border rounded"
                >
                  <option value="active">活跃</option>
                  <option value="capped">额度用尽</option>
                  <option value="banned">已封禁</option>
                  <option value="invalid">无效</option>
                </select>
              </div>
            </div>
            <div className="p-6 border-t border-gray-200 flex gap-2 justify-end">
              <button
                onClick={() => setEditingAccount(null)}
                className="px-4 py-2 bg-gray-200 rounded hover:bg-gray-300"
              >
                取消
              </button>
              <button
                onClick={handleSaveEdit}
                className="px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600"
              >
                保存
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 导入弹窗 */}
      {showImportModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-4xl max-h-[90vh] overflow-y-auto">
            <div className="p-6 border-b border-gray-200">
              <h2 className="text-2xl font-bold">导入账号</h2>
              <p className="text-sm text-gray-500 mt-1">支持多种导入方式</p>
            </div>

            <div className="p-6">
              {/* Tab 切换 */}
              <div className="flex gap-2 mb-6 border-b border-gray-200">
                <button
                  onClick={() => setImportTab("json")}
                  className={`px-4 py-2 border-b-2 transition ${
                    importTab === "json"
                      ? "border-indigo-500 text-indigo-600 font-medium"
                      : "border-transparent text-gray-600 hover:text-gray-900"
                  }`}
                >
                  <FileJson size={16} className="inline mr-2" />
                  JSON 导入
                </button>
                <button
                  onClick={() => setImportTab("kiro")}
                  className={`px-4 py-2 border-b-2 transition ${
                    importTab === "kiro"
                      ? "border-indigo-500 text-indigo-600 font-medium"
                      : "border-transparent text-gray-600 hover:text-gray-900"
                  }`}
                >
                  <Database size={16} className="inline mr-2" />
                  从 Kiro IDE 导入
                </button>
              </div>

              {/* JSON 导入 */}
              {importTab === "json" && (
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">
                      JSON 数据
                    </label>
                    <textarea
                      value={jsonText}
                      onChange={(e) => setJsonText(e.target.value)}
                      className="w-full h-64 px-3 py-2 border rounded font-mono text-sm"
                      placeholder={`[
  {
    "refreshToken": "aor...",
    "provider": "Google"
  }
]`}
                    />
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={handleFileImport}
                      className="px-4 py-2 bg-gray-100 text-gray-700 rounded hover:bg-gray-200 flex items-center gap-2"
                    >
                      <Upload size={16} />
                      从文件导入
                    </button>
                    <div className="flex-1"></div>
                    <button
                      onClick={() => setShowImportModal(false)}
                      className="px-4 py-2 bg-gray-200 rounded hover:bg-gray-300"
                    >
                      取消
                    </button>
                    <button
                      onClick={handleJsonImport}
                      disabled={importing || !jsonText.trim()}
                      className="px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600 disabled:opacity-50 flex items-center gap-2"
                    >
                      {importing ? (
                        <>
                          <RefreshCw size={16} className="animate-spin" />
                          导入中...
                        </>
                      ) : (
                        <>
                          <Upload size={16} />
                          导入
                        </>
                      )}
                    </button>
                  </div>
                </div>
              )}

              {/* Kiro IDE 导入 */}
              {importTab === "kiro" && (
                <div className="space-y-4">
                  <div className="bg-blue-50 border border-blue-200 rounded p-4">
                    <h4 className="font-medium text-blue-900 mb-2">从 Kiro IDE 导入</h4>
                    <p className="text-sm text-blue-700 mb-3">
                      自动检测并导入已在 Kiro IDE 中登录的账号
                    </p>
                    <ul className="text-sm text-blue-700 list-disc list-inside space-y-1">
                      <li>读取本地认证数据库</li>
                      <li>自动获取 refresh_token</li>
                      <li>一键导入账号</li>
                    </ul>
                  </div>
                  <button
                    className="w-full px-4 py-3 bg-gradient-to-r from-indigo-500 to-purple-600 text-white rounded-lg hover:shadow-lg transition flex items-center justify-center gap-2"
                    onClick={async () => {
                      try {
                        await invoke("import_from_kiro_ide");
                        alert("导入成功！");
                        setShowImportModal(false);
                        await loadAccounts();
                      } catch (error: any) {
                        if (error.message?.includes("Tauri API")) {
                          alert(error.message + "\n\n请检查是否在使用 Tauri 桌面窗口运行，而不是浏览器。");
                        } else {
                          alert("导入失败: " + error);
                        }
                      }
                    }}
                  >
                    <Database size={18} />
                    检测并导入 Kiro IDE 账号
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
