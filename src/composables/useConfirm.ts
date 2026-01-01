import { reactive, nextTick } from "vue";

interface ConfirmOptions {
  title?: string;
  description?: string;
  cancelText?: string;
  actionText?: string;
  variant?: "default" | "destructive";
}

interface ConfirmTask {
  options: ConfirmOptions;
  resolve: (value: boolean) => void;
}

// 内部状态（不对外暴露，保持封装性）
const state = reactive({
  isOpen: false,
  title: "",
  description: "",
  cancelText: "取消",
  actionText: "确定",
  variant: "default" as "default" | "destructive",
});

// 任务队列
const queue: ConfirmTask[] = [];
let currentResolve: ((value: boolean) => void) | null = null;

/**
 * 核心调度器：处理队列中的下一个任务
 */
const processQueue = async () => {
  if (state.isOpen || queue.length === 0) return;

  const task = queue.shift()!;

  // 赋值状态
  state.title = task.options.title || "确认操作";
  state.description = task.options.description || "";
  state.cancelText = task.options.cancelText || "取消";
  state.actionText = task.options.actionText || "确定";
  state.variant = task.options.variant || "default";

  currentResolve = task.resolve;

  // 确保 DOM 准备好后开启
  await nextTick();
  state.isOpen = true;
};

/**
 * 暴露给业务使用的 confirm 函数
 */
export const confirm = (options: ConfirmOptions): Promise<boolean> => {
  return new Promise((resolve) => {
    // 将请求推入队列
    queue.push({ options, resolve });
    // 尝试启动处理流程
    processQueue();
  });
};

/**
 * 内部方法：处理点击事件
 */
const handleResult = (result: boolean) => {
  state.isOpen = false;

  // 延迟 resolve，确保弹窗关闭动画完成后再处理逻辑和下一个弹窗
  setTimeout(() => {
    if (currentResolve) {
      currentResolve(result);
      currentResolve = null;
    }
    // 继续处理队列中的下一个
    processQueue();
  }, 300); // 300ms 通常匹配 shadcn (Radix) 的动画时间
};

export { state as confirmState, handleResult };
