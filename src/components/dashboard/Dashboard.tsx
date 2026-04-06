import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/Card";
import { Users, Server, Zap, TrendingUp } from "lucide-react";

interface DashboardStats {
  totalAccounts: number;
  activeAccounts: number;
  totalProviders: number;
  activeProvider?: string;
  gatewayRunning: boolean;
  gatewayPort?: number;
}

export function Dashboard() {
  const [stats, setStats] = useState<DashboardStats>({
    totalAccounts: 0,
    activeAccounts: 0,
    totalProviders: 0,
    gatewayRunning: false,
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadStats();
  }, []);

  const loadStats = async () => {
    try {
      setLoading(true);
      const accounts = await invoke<any[]>("get_accounts");
      const providersData = await invoke<any>("get_providers");
      const gatewayConfig = await invoke<any>("get_gateway_config");

      setStats({
        totalAccounts: accounts.length,
        activeAccounts: accounts.filter((a: any) => a.status === "active").length,
        totalProviders: providersData.providers.length,
        activeProvider: providersData.active_provider?.name,
        gatewayRunning: gatewayConfig.isRunning,
        gatewayPort: gatewayConfig.port,
      });
    } catch (error) {
      console.error("Failed to load stats:", error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div className="text-center py-12">加载中...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">仪表盘</h1>
        <button
          onClick={loadStats}
          className="px-4 py-2 bg-gray-100 rounded hover:bg-gray-200"
        >
          刷新
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">总账号数</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalAccounts}</div>
            <p className="text-xs text-muted-foreground">
              {stats.activeAccounts} 个活跃
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">供应商数</CardTitle>
            <Zap className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalProviders}</div>
            <p className="text-xs text-muted-foreground">
              {stats.activeProvider ? `当前: ${stats.activeProvider}` : "未设置"}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">网关服务</CardTitle>
            <Server className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats.gatewayRunning ? "运行中" : "已停止"}
            </div>
            <p className="text-xs text-muted-foreground">
              {stats.gatewayPort ? `端口 ${stats.gatewayPort}` : "未配置"}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">额度使用</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">--</div>
            <p className="text-xs text-muted-foreground">
              同步账号后显示
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>快速操作</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <button className="w-full px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600">
              添加账号
            </button>
            <button className="w-full px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600">
              添加供应商
            </button>
            <button className="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600">
              启动网关
            </button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>最近活动</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-center text-gray-500 py-8">
              暂无活动记录
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>系统信息</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-gray-600">版本:</span>
              <span className="ml-2 font-medium">0.1.0</span>
            </div>
            <div>
              <span className="text-gray-600">数据库:</span>
              <span className="ml-2 font-medium">SQLite</span>
            </div>
            <div>
              <span className="text-gray-600">框架:</span>
              <span className="ml-2 font-medium">Tauri 2.0</span>
            </div>
            <div>
              <span className="text-gray-600">前端:</span>
              <span className="ml-2 font-medium">React + TypeScript</span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
