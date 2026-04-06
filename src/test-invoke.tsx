import { useEffect, useState } from "react";

export function TestInvoke() {
  const [result, setResult] = useState<string>("");

  useEffect(() => {
    const testInvoke = async () => {
      try {
        // 检查 window 对象
        const hasTauriInternals = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
        const hasTauri = typeof window !== 'undefined' && '__TAURI__' in window;
        const windowKeys = typeof window !== 'undefined' ? Object.keys(window).filter(k => k.includes('TAURI')).join(', ') : 'no window';
        
        // 动态导入测试
        const coreModule = await import("@tauri-apps/api/core");
        const hasInvoke = typeof coreModule.invoke === "function";
        
        let details = "";
        if (hasTauriInternals) {
          details = `window.__TAURI_INTERNALS__: 存在\nwindow.__TAURI_INTERNALS__.invoke: ${typeof window.__TAURI_INTERNALS__.invoke}`;
        } else {
          details = `window.__TAURI_INTERNALS__: 不存在\n这是问题所在!`;
        }

        setResult(`
窗口环境检查:
- window.__TAURI__: ${hasTauri ? '存在' : '不存在'}
- window.__TAURI_INTERNALS__: ${hasTauriInternals ? '存在' : '不存在'}
- Tauri 相关 window 键: ${windowKeys || '无'}

模块加载: 成功
invoke 类型: ${typeof coreModule.invoke}
invoke 可用: ${hasInvoke ? "是" : "否"}

详细:
${details}
        `);

        if (hasInvoke && hasTauriInternals) {
          try {
            const accounts = await coreModule.invoke("get_accounts");
            setResult(prev => prev + `\n\n调用结果: 成功获取 ${Array.isArray(accounts) ? accounts.length : "未知"} 个账号`);
          } catch (e) {
            setResult(prev => prev + `\n\n调用失败: ${e}`);
          }
        }
      } catch (e) {
        setResult(`模块加载失败: ${e}`);
      }
    };

    testInvoke();
  }, []);

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-4">Invoke 测试</h1>
      <pre className="bg-gray-100 p-4 rounded whitespace-pre-wrap">{result}</pre>
    </div>
  );
}
