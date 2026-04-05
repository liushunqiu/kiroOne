import { NavLink } from "react-router-dom";
import { LayoutDashboard, Users, Server, Settings } from "lucide-react";

const navItems = [
  { icon: LayoutDashboard, label: "仪表盘", path: "/" },
  { icon: Users, label: "账号管理", path: "/accounts" },
  { icon: Server, label: "API 网关", path: "/gateway" },
  { icon: Settings, label: "设置", path: "/settings" },
];

export function Sidebar() {
  return (
    <aside className="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
      <div className="p-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
          AI Unified
        </h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
          账号管理与 API 网关
        </p>
      </div>

      <nav className="px-4">
        {navItems.map((item) => (
          <NavLink
            key={item.path}
            to={item.path}
            className={({ isActive }) =>
              `flex items-center gap-3 px-4 py-3 rounded-lg mb-2 transition-colors ${
                isActive
                  ? "bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400"
                  : "text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50"
              }`
            }
          >
            <item.icon className="w-5 h-5" />
            <span className="font-medium">{item.label}</span>
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
