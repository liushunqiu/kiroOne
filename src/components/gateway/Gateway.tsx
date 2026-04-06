import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "../ui/Card";
import { Play, Square, Copy, Key } from "lucide-react";

interface GatewayConfig {
  id: string;
  port: number;
  apiKey: string;
  isRunning: boolean;
  defaultModel?: string;
  proxyEnabled: boolean;
  proxyUrl?: string;
  createdAt: string;
  updatedAt: string;
}

export function Gateway() {
  const [config, setConfig] = useState<GatewayConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setLoading(true);
      const data = await invoke<GatewayConfig>("get_gateway_config");
      setConfig(data);
    } catch (error) {
      console.error("Failed to load gateway config:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!config) return;

    try {
      setSaving(true);
      await invoke("update_gateway_config", {
        port: config.port,
        apiKey: config.apiKey,
        defaultModel: config.defaultModel,
      });
      alert("保存成功");
    } catch (error) {
      console.error("Failed to save:", error);
      alert("保存失败");
    } finally {
      setSaving(false);
    }
  };

  const handleToggleServer = async () => {
    if (!config) return;

    try {
      if (config.isRunning) {
        // TODO: 实现停止服务器
        alert("停止服务器功能待实现");
      } else {
        // TODO: 实现启动服务器
        alert("启动服务器功能待实现");
      }
    } catch (error) {
      console.error("Failed to toggle server:", error);
      alert("操作失败");
    }
  };

  const copyApiKey = () => {
    if (!config) return;
    navigator.clipboard.writeText(config.apiKey);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const generateApiKey = () => {
    if (!config) return;
    const newKey = "sk-" + Array.from({ length: 32 }, () => 
      Math.random().toString(36).charAt(2)
    ).join("");
    setConfig({ ...config, apiKey: newKey });
  };

  if (loading) {
    return <div className="text-center py-8">加载中...</div>;
  }

  if (!config) {
    return <div className="text-center py-8 text-red-500">加载配置失败</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">API 网关</h1>
        <button
          onClick={handleToggleServer}
          className={`px-6 py-2 rounded flex items-center gap-2 ${
            config.isRunning
              ? "bg-red-500 hover:bg-red-600 text-white"
              : "bg-green-500 hover:bg-green-600 text-white"
          }`}
        >
          {config.isRunning ? (
            <>
              <Square size={16} />
              停止服务
            </>
          ) : (
            <>
              <Play size={16} />
              启动服务
            </>
          )}
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>服务配置</CardTitle>
            <CardDescription>配置 API 网关服务参数</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">端口号</label>
              <input
                type="number"
                value={config.port}
                onChange={(e) => setConfig({ ...config, port: parseInt(e.target.value) })}
                className="w-full px-3 py-2 border rounded"
                min="1024"
                max="65535"
              />
              <p className="text-xs text-gray-500 mt-1">范围: 1024-65535</p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">默认模型</label>
              <input
                type="text"
                value={config.defaultModel || ""}
                onChange={(e) => setConfig({ ...config, defaultModel: e.target.value })}
                className="w-full px-3 py-2 border rounded"
                placeholder="例如: claude-sonnet-4"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">代理设置</label>
              <div className="flex items-center gap-2 mb-2">
                <input
                  type="checkbox"
                  id="proxyEnabled"
                  checked={config.proxyEnabled}
                  onChange={(e) => setConfig({ ...config, proxyEnabled: e.target.checked })}
                />
                <label htmlFor="proxyEnabled">启用代理</label>
              </div>
              {config.proxyEnabled && (
                <input
                  type="text"
                  value={config.proxyUrl || ""}
                  onChange={(e) => setConfig({ ...config, proxyUrl: e.target.value })}
                  className="w-full px-3 py-2 border rounded"
                  placeholder="http://127.0.0.1:7890"
                />
              )}
            </div>

            <button
              onClick={handleSave}
              disabled={saving}
              className="w-full px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600 disabled:opacity-50"
            >
              {saving ? "保存中..." : "保存配置"}
            </button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>API 密钥</CardTitle>
            <CardDescription>用于访问 API 服务的密钥</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">API Key</label>
              <div className="flex gap-2">
                <input
                  type="password"
                  value={config.apiKey}
                  readOnly
                  className="flex-1 px-3 py-2 border rounded bg-gray-50"
                />
                <button
                  onClick={copyApiKey}
                  className="px-3 py-2 bg-gray-100 hover:bg-gray-200 rounded"
                  title="复制"
                >
                  <Copy size={16} />
                </button>
                <button
                  onClick={generateApiKey}
                  className="px-3 py-2 bg-gray-100 hover:bg-gray-200 rounded"
                  title="生成新密钥"
                >
                  <Key size={16} />
                </button>
              </div>
              {copied && (
                <p className="text-xs text-green-600 mt-1">已复制到剪贴板</p>
              )}
            </div>

            <div className="bg-blue-50 border border-blue-200 rounded p-4">
              <h4 className="font-medium text-blue-900 mb-2">使用示例</h4>
              <div className="bg-gray-900 text-green-400 p-3 rounded text-xs overflow-x-auto">
                <p>curl http://localhost:{config.port}/v1/chat/completions \</p>
                <p>&nbsp;&nbsp;-H "Authorization: Bearer YOUR_API_KEY" \</p>
                <p>&nbsp;&nbsp;-H "Content-Type: application/json" \</p>
                <p>&nbsp;&nbsp;-d &#39;&#123;&quot;model&quot;: &quot;claude-sonnet-4&quot;, &quot;messages&quot;: [...]&#125;&#39;</p>
              </div>
            </div>

            <div className="bg-gray-50 rounded p-4">
              <h4 className="font-medium mb-2">API 端点</h4>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <code className="text-xs">GET /health</code>
                  <span className="text-gray-600">健康检查</span>
                </div>
                <div className="flex justify-between">
                  <code className="text-xs">GET /v1/models</code>
                  <span className="text-gray-600">模型列表</span>
                </div>
                <div className="flex justify-between">
                  <code className="text-xs">POST /v1/chat/completions</code>
                  <span className="text-gray-600">OpenAI 兼容聊天</span>
                </div>
                <div className="flex justify-between">
                  <code className="text-xs">POST /v1/messages</code>
                  <span className="text-gray-600">Anthropic 兼容消息</span>
                </div>
                <div className="flex justify-between">
                  <code className="text-xs">GET /usage</code>
                  <span className="text-gray-600">额度查询</span>
                </div>
                <div className="flex justify-between">
                  <code className="text-xs">GET /account</code>
                  <span className="text-gray-600">账号信息</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>服务状态</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="bg-gray-50 rounded p-4">
              <div className="text-sm text-gray-600 mb-1">服务状态</div>
              <div className={`text-lg font-bold ${config.isRunning ? "text-green-600" : "text-gray-400"}`}>
                {config.isRunning ? "运行中" : "已停止"}
              </div>
            </div>
            <div className="bg-gray-50 rounded p-4">
              <div className="text-sm text-gray-600 mb-1">监听端口</div>
              <div className="text-lg font-bold">{config.port}</div>
            </div>
            <div className="bg-gray-50 rounded p-4">
              <div className="text-sm text-gray-600 mb-1">API 地址</div>
              <div className="text-lg font-bold">localhost:{config.port}</div>
            </div>
            <div className="bg-gray-50 rounded p-4">
              <div className="text-sm text-gray-600 mb-1">最后更新</div>
              <div className="text-sm font-medium">{config.updatedAt}</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
