import { Card } from "../ui/Card";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Account {
  id: string;
  name: string;
  email: string;
  account_type: "Social" | "IdC";
  refresh_token: string;
  access_token?: string;
  token_expires_at?: string;
  quota?: Quota;
  tags: string[];
  created_at: string;
  updated_at: string;
  is_active: boolean;
}

interface Quota {
  main_quota: number;
  main_quota_max: number;
  trial_quota: number;
  trial_quota_max: number;
  bonus_quota: number;
  bonus_quota_max: number;
}

export function Accounts() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(false);
  const [showModal, setShowModal] = useState(false);
  const [formData, setFormData] = useState({
    name: "",
    email: "",
    refresh_token: "",
  });

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

  // 导入账号
  const handleImportAccount = async () => {
    if (!formData.name || !formData.email || !formData.refresh_token) {
      alert("请填写所有必填字段");
      return;
    }

    try {
      const newAccount: Omit<Account, "id" | "created_at" | "updated_at" | "is_active"> = {
        name: formData.name,
        email: formData.email,
        account_type: "Social",
        refresh_token: formData.refresh_token,
        tags: [],
      };

      await invoke("import_account", { account: newAccount });
      
      // 重置表单并关闭对话框
      setFormData({ name: "", email: "", refresh_token: "" });
      setShowModal(false);
      
      // 重新加载账号列表
      await loadAccounts();
      
      alert("账号导入成功！");
    } catch (error) {
      console.error("Failed to import account:", error);
      alert(`导入失败: ${error}`);
    }
  };

  // 切换活跃账号
  const handleSwitchAccount = async (accountId: string) => {
    try {
      await invoke("switch_account", { accountId });
      await loadAccounts();
    } catch (error) {
      console.error("Failed to switch account:", error);
      alert(`切换失败: ${error}`);
    }
  };

  // 删除账号
  const handleDeleteAccount = async (accountId: string) => {
    if (!confirm("确定要删除这个账号吗？")) {
      return;
    }

    try {
      await invoke("delete_account", { accountId });
      await loadAccounts();
    } catch (error) {
      console.error("Failed to delete account:", error);
      alert(`删除失败: ${error}`);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
          账号管理
        </h1>
        <button
          onClick={() => setShowModal(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          导入账号
        </button>
      </div>

      {accounts.length === 0 ? (
        <Card>
          <p className="text-gray-500 dark:text-gray-400 text-center py-8">
            暂无账号，请点击「导入账号」添加
          </p>
        </Card>
      ) : (
        <div className="space-y-4">
          {accounts.map((account) => (
            <Card key={account.id}>
              <div className="flex justify-between items-start">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                      {account.name}
                    </h3>
                    {account.is_active && (
                      <span className="px-2 py-1 text-xs bg-green-100 text-green-700 rounded">
                        活跃
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    {account.email}
                  </p>
                </div>
                <div className="flex gap-2">
                  {!account.is_active && (
                    <button
                      onClick={() => handleSwitchAccount(account.id)}
                      className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
                    >
                      设为活跃
                    </button>
                  )}
                  <button
                    onClick={() => handleDeleteAccount(account.id)}
                    className="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700"
                  >
                    删除
                  </button>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* 导入账号对话框 */}
      {showModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md">
            <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
              导入 Kiro 账号
            </h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  账号名称 *
                </label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  placeholder="例如：我的主账号"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  邮箱地址 *
                </label>
                <input
                  type="email"
                  value={formData.email}
                  onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  placeholder="your@email.com"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Refresh Token *
                </label>
                <textarea
                  value={formData.refresh_token}
                  onChange={(e) => setFormData({ ...formData, refresh_token: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white h-32"
                  placeholder="从 Kiro 获取的 refresh_token..."
                />
                <p className="text-xs text-gray-500 mt-1">
                  如何获取？登录 Kiro 后，从浏览器开发者工具的 Application/Storage 中查找
                </p>
              </div>
            </div>

            <div className="flex gap-3 mt-6">
              <button
                onClick={handleImportAccount}
                disabled={loading}
                className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
              >
                {loading ? "导入中..." : "确认导入"}
              </button>
              <button
                onClick={() => setShowModal(false)}
                className="px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600"
              >
                取消
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
