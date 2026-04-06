// Tauri invoke 包装器 - 兼容 Tauri 2.0
import { isTauri } from "@tauri-apps/api/core";

let invokeFn: any = null;

// 动态导入 invoke
async function getInvoke() {
  if (invokeFn) return invokeFn;
  
  if (!isTauri()) {
    throw new Error("Tauri 环境检测失败。请确保:\n1. 正在使用 Tauri 桌面窗口运行(不是在浏览器中)\n2. 应用已正确启动");
  }
  
  const core = await import("@tauri-apps/api/core");
  invokeFn = core.invoke;
  return invokeFn;
}

export const invoke = async (cmd: string, args?: any) => {
  const fn = await getInvoke();
  return fn(cmd, args);
};

export { isTauri };
