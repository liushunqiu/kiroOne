import { Card } from "../ui/Card";
import { Copy } from "lucide-react";

export function Gateway() {
  const apiUrl = "http://127.0.0.1:8000";

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
        API 网关
      </h1>

      <Card>
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          网关状态
        </h2>
        <div className="flex items-center gap-3">
          <div className="w-3 h-3 bg-green-500 rounded-full animate-pulse" />
          <span className="text-green-600 dark:text-green-400 font-medium">
            运行中
          </span>
        </div>
      </Card>

      <Card>
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          API 端点
        </h2>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              OpenAI 兼容
            </label>
            <div className="flex items-center gap-2">
              <code className="flex-1 bg-gray-100 dark:bg-gray-700 px-4 py-2 rounded text-sm">
                {apiUrl}/v1/chat/completions
              </code>
              <button className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
                <Copy className="w-4 h-4" />
              </button>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Anthropic 兼容
            </label>
            <div className="flex items-center gap-2">
              <code className="flex-1 bg-gray-100 dark:bg-gray-700 px-4 py-2 rounded text-sm">
                {apiUrl}/v1/messages
              </code>
              <button className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
                <Copy className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </Card>

      <Card>
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          Claude Code 配置
        </h2>
        <div className="space-y-3">
          <p className="text-gray-700 dark:text-gray-300">
            在 Claude Code 中设置以下环境变量:
          </p>
          <code className="block bg-gray-100 dark:bg-gray-800 px-4 py-3 rounded text-sm space-y-1">
            <div>export ANTHROPIC_BASE_URL={apiUrl}</div>
            <div>export ANTHROPIC_API_KEY=sk-local-kiro-gateway</div>
          </code>
          <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
            一键配置 Claude Code
          </button>
        </div>
      </Card>
    </div>
  );
}
