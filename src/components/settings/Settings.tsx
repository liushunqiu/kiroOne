import { Card } from "../ui/Card";

export function Settings() {
  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
        设置
      </h1>

      <Card>
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          网关设置
        </h2>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              端口
            </label>
            <input
              type="number"
              defaultValue={8000}
              className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium text-gray-900 dark:text-white">
                自动启动
              </p>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                应用启动时自动启动 API 网关
              </p>
            </div>
            <input type="checkbox" defaultChecked className="w-5 h-5" />
          </div>
        </div>
      </Card>

      <Card>
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          代理设置
        </h2>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium text-gray-900 dark:text-white">
                启用代理
              </p>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                使用代理服务器连接 Kiro API
              </p>
            </div>
            <input type="checkbox" className="w-5 h-5" />
          </div>
        </div>
      </Card>
    </div>
  );
}
