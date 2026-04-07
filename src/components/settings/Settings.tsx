import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "../ui/Card";
import { Plus, RefreshCw, Trash2, Edit, Zap, CheckCircle } from "lucide-react";

interface Provider {
  id: string;
  name: string;
  apiBaseUrl: string;
  apiKey: string;
  apiFormat: string;
  models?: string;
  isActive: boolean;
  accountId?: string;
  createdAt: string;
  updatedAt: string;
}

interface ProviderPreset {
  id: string;
  name: string;
  apiBaseUrl: string;
  apiFormat: string;
  description: string;
}

export function Settings() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [activeProvider, setActiveProvider] = useState<Provider | null>(null);
  const [presets, setPresets] = useState<ProviderPreset[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [switchingId, setSwitchingId] = useState<string | null>(null);
  const [editingProvider, setEditingProvider] = useState<Provider | null>(null);

  const [newProvider, setNewProvider] = useState({
    name: "",
    apiBaseUrl: "",
    apiKey: "",
    apiFormat: "anthropic",
  });

  useEffect(() => {
    loadData();
    loadPresets();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const data = await invoke<any>("get_providers");
      setProviders(data.providers);
      setActiveProvider(data.active_provider);
    } catch (error) {
      console.error("Failed to load providers:", error);
    } finally {
      setLoading(false);
    }
  };

  const loadPresets = async () => {
    try {
      const data = await invoke<ProviderPreset[]>("get_provider_presets");
      setPresets(data);
    } catch (error) {
      console.error("Failed to load presets:", error);
    }
  };

  const handleAddProvider = async () => {
    await handleSaveProvider();
  };

  const handleDeleteProvider = async (id: string) => {
    if (!confirm("确定要删除这个供应商吗?")) return;

    try {
      await invoke("delete_provider", { id });
      await loadData();
    } catch (error) {
      console.error("Failed to delete provider:", error);
      alert("删除失败");
    }
  };

  const handleSwitchProvider = async (id: string) => {
    if (switchingId) return;

    if (!confirm("确定要切换到这个供应商吗?这将更新 Claude Code 的配置")) return;

    try {
      setSwitchingId(id);
      await invoke("switch_provider", { id });
      await loadData();
      alert("切换成功");
    } catch (error) {
      console.error("Failed to switch provider:", error);
      alert("切换失败");
    } finally {
      setSwitchingId(null);
    }
  };

  const handleUsePreset = (preset: ProviderPreset) => {
    setNewProvider({
      name: preset.name,
      apiBaseUrl: preset.apiBaseUrl,
      apiKey: "",
      apiFormat: preset.apiFormat,
    });
    setShowAddForm(true);
  };

  const handleEditProvider = (provider: Provider) => {
    setEditingProvider(provider);
    setNewProvider({
      name: provider.name,
      apiBaseUrl: provider.apiBaseUrl,
      apiKey: provider.apiKey,
      apiFormat: provider.apiFormat,
    });
    setShowAddForm(true);
  };

  const handleSaveProvider = async () => {
    if (!newProvider.name || !newProvider.apiBaseUrl || !newProvider.apiKey) {
      alert("请填写完整信息");
      return;
    }

    try {
      if (editingProvider) {
        // 更新现有供应商
        await invoke("update_provider", {
          id: editingProvider.id,
          name: newProvider.name,
          apiBaseUrl: newProvider.apiBaseUrl,
          apiKey: newProvider.apiKey,
          apiFormat: newProvider.apiFormat,
        });
      } else {
        // 添加新供应商
        await invoke("add_provider", {
          name: newProvider.name,
          apiBaseUrl: newProvider.apiBaseUrl,
          apiKey: newProvider.apiKey,
          apiFormat: newProvider.apiFormat,
        });
      }
      setShowAddForm(false);
      setEditingProvider(null);
      setNewProvider({ name: "", apiBaseUrl: "", apiKey: "", apiFormat: "anthropic" });
      await loadData();
    } catch (error) {
      console.error("Failed to save provider:", error);
      alert("保存失败");
    }
  };

  const getFormatLabel = (format: string) => {
    switch (format) {
      case "anthropic":
        return "Anthropic";
      case "openai_chat":
        return "OpenAI Chat";
      case "openai_responses":
        return "OpenAI Responses";
      default:
        return format;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">供应商管理</h1>
        <button
          onClick={() => setShowAddForm(!showAddForm)}
          className="px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600 flex items-center gap-2"
        >
          <Plus size={16} />
          添加供应商
        </button>
      </div>

      {activeProvider && (
        <Card className="border-2 border-green-500 bg-green-50">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <CheckCircle className="text-green-600" size={20} />
              当前活跃供应商
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <div className="text-sm text-gray-600">名称</div>
                <div className="font-medium">{activeProvider.name}</div>
              </div>
              <div>
                <div className="text-sm text-gray-600">API 地址</div>
                <div className="font-mono text-sm">{activeProvider.apiBaseUrl}</div>
              </div>
              <div>
                <div className="text-sm text-gray-600">API 格式</div>
                <div>{getFormatLabel(activeProvider.apiFormat)}</div>
              </div>
              <div>
                <div className="text-sm text-gray-600">更新时间</div>
                <div className="text-sm">{activeProvider.updatedAt}</div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {showAddForm && (
        <Card>
          <CardHeader>
            <CardTitle>{editingProvider ? "编辑供应商" : "添加供应商"}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">名称</label>
              <input
                type="text"
                value={newProvider.name}
                onChange={(e) => setNewProvider({ ...newProvider, name: e.target.value })}
                className="w-full px-3 py-2 border rounded"
                placeholder="例如: My Provider"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">API Base URL</label>
              <input
                type="text"
                value={newProvider.apiBaseUrl}
                onChange={(e) => setNewProvider({ ...newProvider, apiBaseUrl: e.target.value })}
                className="w-full px-3 py-2 border rounded"
                placeholder="https://api.example.com"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">API Key</label>
              <input
                type="password"
                value={newProvider.apiKey}
                onChange={(e) => setNewProvider({ ...newProvider, apiKey: e.target.value })}
                className="w-full px-3 py-2 border rounded"
                placeholder="sk-..."
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">API 格式</label>
              <select
                value={newProvider.apiFormat}
                onChange={(e) => setNewProvider({ ...newProvider, apiFormat: e.target.value })}
                className="w-full px-3 py-2 border rounded"
              >
                <option value="anthropic">Anthropic</option>
                <option value="openai_chat">OpenAI Chat</option>
                <option value="openai_responses">OpenAI Responses</option>
              </select>
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleAddProvider}
                className="flex-1 px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600"
              >
                {editingProvider ? "保存" : "添加"}
              </button>
              <button
                onClick={() => {
                  setShowAddForm(false);
                  setEditingProvider(null);
                  setNewProvider({ name: "", apiBaseUrl: "", apiKey: "", apiFormat: "anthropic" });
                }}
                className="px-4 py-2 bg-gray-200 rounded hover:bg-gray-300"
              >
                取消
              </button>
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <span>供应商列表 ({providers.length})</span>
            <button
              onClick={loadData}
              className="px-3 py-1 text-sm bg-gray-100 rounded hover:bg-gray-200 flex items-center gap-1"
            >
              <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
              刷新
            </button>
          </CardTitle>
        </CardHeader>
        <CardContent>
          {loading && providers.length === 0 ? (
            <div className="text-center py-8">加载中...</div>
          ) : providers.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              暂无供应商,请先添加
            </div>
          ) : (
            <div className="space-y-4">
              {providers.map((provider) => (
                <div
                  key={provider.id}
                  className={`p-4 border rounded ${
                    provider.isActive ? "border-green-500 bg-green-50" : ""
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <h3 className="font-medium">{provider.name}</h3>
                        {provider.isActive && (
                          <span className="px-2 py-1 bg-green-500 text-white text-xs rounded">
                            活跃
                          </span>
                        )}
                      </div>
                      <div className="text-sm text-gray-600 space-y-1">
                        <div>API: {provider.apiBaseUrl}</div>
                        <div>格式: {getFormatLabel(provider.apiFormat)}</div>
                        <div>更新: {provider.updatedAt}</div>
                      </div>
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleSwitchProvider(provider.id)}
                        disabled={switchingId === provider.id || provider.isActive}
                        className="p-2 text-green-600 hover:bg-green-50 rounded disabled:opacity-50"
                        title="切换到此供应商"
                      >
                        <Zap
                          size={18}
                          className={switchingId === provider.id ? "animate-spin" : ""}
                        />
                      </button>
                      <button
                        onClick={() => handleEditProvider(provider)}
                        className="p-2 text-blue-600 hover:bg-blue-50 rounded"
                        title="编辑"
                      >
                        <Edit size={18} />
                      </button>
                      <button
                        onClick={() => handleDeleteProvider(provider.id)}
                        className="p-2 text-red-600 hover:bg-red-50 rounded"
                        title="删除"
                      >
                        <Trash2 size={18} />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {presets.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>预设供应商</CardTitle>
            <CardDescription>快速添加常用的供应商配置</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {presets.map((preset) => (
                <div
                  key={preset.id}
                  className="p-4 border rounded hover:border-indigo-500 cursor-pointer transition"
                  onClick={() => handleUsePreset(preset)}
                >
                  <h4 className="font-medium mb-1">{preset.name}</h4>
                  <p className="text-sm text-gray-600 mb-2">{preset.description}</p>
                  <div className="text-xs text-gray-500">{preset.apiBaseUrl}</div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
