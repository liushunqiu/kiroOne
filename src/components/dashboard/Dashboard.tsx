import { Card } from "../ui/Card";
import { Server, Users, Activity } from "lucide-react";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Account {
  id: string;
  name: string;
  email: string;
  is_active: boolean;
}

export function Dashboard() {
  const [activeAccounts, setActiveAccounts] = useState<number>(0);
  const [gatewayRunning, setGatewayRunning] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    async function fetchData() {
      try {
        // 获取账号列表
        const accounts: Account[] = await invoke("get_accounts");
        const activeCount = accounts.filter(acc => acc.is_active).length;
        setActiveAccounts(activeCount);

        // 获取网关状态
        const status = await invoke("get_gateway_status");
        setGatewayRunning(status.running);
      } catch (error) {
        console.error("Failed to fetch dashboard data:", error);
      } finally {
        setLoading(false);
      }
    }

    fetchData();
  }, []);

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
        仪表盘
      </h1>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card>
          <div className="flex items-center gap-4">
            <div className="p-3 bg-blue-100 dark:bg-blue-900/30 rounded-lg">
              <Server className="w-6 h-6 text-blue-600 dark:text-blue-400" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                API 网关状态
              </p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                {loading ? "加载中..." : gatewayRunning ? "运行中" : "已停止"}
              </p>
            </div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-4">
            <div className="p-3 bg-green-100 dark:bg-green-900/30 rounded-lg">
              <Users className="w-6 h-6 text-green-600 dark:text-green-400" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                活跃账号
              </p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                {loading ? "-" : activeAccounts}
              </p>
            </div>
          </div>
        </Card>

        <Card>
          <div className="flex items-center gap-4">
            <div className="p-3 bg-purple-100 dark:bg-purple-900/30 rounded-lg">
              <Activity className="w-6 h-6 text-purple-600 dark:text-purple-400" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                今日请求
              </p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                0
              </p>
            </div>
          </div>
        </Card>
      </div>

      <Card>
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          快速开始
        </h2>
        <div className="space-y-3 text-gray-700 dark:text-gray-300">
          <p>1. 在「账号管理」中导入你的 Kiro 账号</p>
          <p>2. 在「API 网关」中查看 API 地址</p>
          <p>
            3. 在 Claude Code 中配置:
            <code className="block bg-gray-100 dark:bg-gray-800 px-3 py-2 rounded mt-2 text-sm">
              export ANTHROPIC_BASE_URL=http://127.0.0.1:8000
              <br />
              export ANTHROPIC_API_KEY=sk-local-kiro-gateway
            </code>
          </p>
        </div>
      </Card>
    </div>
  );
}
