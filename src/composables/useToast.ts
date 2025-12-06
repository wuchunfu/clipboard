import { ref } from "vue";

const toastMessage = ref<string | null>(null);
let toastTimeout: number | null = null;

export function useToast() {
  function showToast(msg: string) {
    if (toastTimeout) clearTimeout(toastTimeout);
    toastMessage.value = msg;
    toastTimeout = window.setTimeout(() => {
      toastMessage.value = null;
    }, 2000);
  }

  return {
    toastMessage,
    showToast,
  };
}
