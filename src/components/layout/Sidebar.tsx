import { Link, useLocation } from "react-router-dom";
import { LayoutDashboard, Users, Server, Settings, Bug } from "lucide-react";

const navItems = [
  { path: "/", icon: LayoutDashboard, label: "仪表盘" },
  { path: "/accounts", icon: Users, label: "账号管理" },
  { path: "/gateway", icon: Server, label: "API 网关" },
  { path: "/settings", icon: Settings, label: "供应商设置" },
  { path: "/test", icon: Bug, label: "测试" },
];

export function Sidebar() {
  const location = useLocation();

  return (
    <aside className="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col">
      <div className="p-6 border-b border-gray-200 dark:border-gray-700">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
          Kiro One
        </h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
          统一管理平台
        </p>
      </div>

      <nav className="flex-1 p-4 space-y-2">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path;
          return (
            <Link
              key={item.path}
              to={item.path}
              className={`flex items-center gap-3 px-4 py-3 rounded-lg transition ${
                isActive
                  ? "bg-indigo-500 text-white"
                  : "text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
              }`}
            >
              <item.icon size={20} />
              <span className="font-medium">{item.label}</span>
            </Link>
          );
        })}
      </nav>

      <div className="p-4 border-t border-gray-200 dark:border-gray-700">
        <div className="text-xs text-gray-500 dark:text-gray-400">
          v0.1.0
        </div>
      </div>
    </aside>
  );
}
