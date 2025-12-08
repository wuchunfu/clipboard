import { ref, computed, nextTick, watch } from "vue";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "vue-i18n";
import { useToast } from "./useToast";
import type { ClipboardItem } from "../types";

export function useClipboard() {
  const { t } = useI18n();
  const { showToast } = useToast();

  const history = ref<ClipboardItem[]>([]);
  const searchQuery = ref("");
  const selectedIndex = ref(0);
  const activeFilter = ref<"all" | "text" | "image" | "sensitive">("all");
  const previewItem = ref<ClipboardItem | null>(null);
  const previewContent = ref("");

  watch(previewItem, async (newItem) => {
    if (!newItem) {
      previewContent.value = "";
      return;
    }

    if (newItem.kind === "text" && newItem.id) {
      try {
        // Initially set to truncated content to show something immediately
        previewContent.value = newItem.content;
        // Fetch full content
        const fullContent = await invoke<string>("get_item_content", {
          id: newItem.id,
        });
        // Only update if the preview item hasn't changed in the meantime
        if (previewItem.value?.id === newItem.id) {
          previewContent.value = fullContent;
        }
      } catch (e) {
        console.error("Failed to fetch full content for preview:", e);
      }
    } else {
      previewContent.value = "";
    }
  });

  const filteredHistory = computed(() => {
    let items = history.value;

    // Filter by Type
    if (activeFilter.value === "text") {
      items = items.filter((i) => i.kind === "text");
    } else if (activeFilter.value === "image") {
      items = items.filter((i) => i.kind === "image");
    } else if (activeFilter.value === "sensitive") {
      items = items.filter((i) => i.is_sensitive);
    }

    // Filter by Search
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      items = items.filter((item) => {
        if (item.kind === "text") {
          return item.content.toLowerCase().includes(query);
        }
        return false;
      });
    }
    return items;
  });

  async function loadHistory() {
    try {
      history.value = await invoke<ClipboardItem[]>("get_history", {
        page: 1,
        pageSize: 1000,
      });
      // Ensure selection is valid
      if (selectedIndex.value >= filteredHistory.value.length) {
        selectedIndex.value = 0;
      }
    } catch (e) {
      console.error("Failed to load history:", e);
    }
  }

  async function pasteItem(item: ClipboardItem, hideWindow = true) {
    try {
      // Close preview if open
      previewItem.value = null;

      if (hideWindow) {
        await getCurrentWindow().hide();
      }

      // Fetch full content if it's text and might be truncated
      let content = item.content;
      if (item.kind === "text" && item.id) {
        try {
          content = await invoke<string>("get_item_content", { id: item.id });
        } catch (e) {
          console.error("Failed to fetch full content, using preview:", e);
        }
      }

      await invoke("set_clipboard_item", {
        content: content,
        kind: item.kind,
      });
      await loadHistory();
      searchQuery.value = "";
      showToast(t("toast.copied"));
    } catch (e) {
      console.error("Failed to set clipboard item:", e);
    }
  }

  async function deleteItem(index: number) {
    const item = filteredHistory.value[index];
    const realIndex = history.value.indexOf(item);

    if (realIndex !== -1) {
      try {
        await invoke("delete_item", { index: realIndex });
        await loadHistory();
        showToast(t("toast.deleted"));
      } catch (e) {
        console.error("Failed to delete item:", e);
      }
    }
  }

  async function toggleSensitive(index: number) {
    const item = filteredHistory.value[index];
    const realIndex = history.value.indexOf(item);

    if (realIndex !== -1) {
      try {
        const newState = await invoke<boolean>("toggle_sensitive", {
          index: realIndex,
        });
        history.value[realIndex].is_sensitive = newState as boolean;
        showToast(
          newState ? t("toast.markedSensitive") : t("toast.unmarkedSensitive")
        );
      } catch (e) {
        console.error("Failed to toggle sensitive:", e);
      }
    }
  }

  async function clearHistory() {
    try {
      await invoke("clear_history");
      await loadHistory();
      showToast(t("toast.historyCleared"));
    } catch (e) {
      console.error("Failed to clear history:", e);
    }
  }

  function getImageSrc(content: string) {
    if (content.startsWith("/") || content.match(/^[a-zA-Z]:\\/)) {
      return convertFileSrc(content);
    }
    return `data:image/png;base64,${content}`;
  }

  function scrollToSelected() {
    nextTick(() => {
      const el = document.querySelector(".selected-item");
      if (el) {
        el.scrollIntoView({ block: "nearest" });
      }
    });
  }

  // Setup listeners
  async function setupClipboardListeners() {
    await listen("clipboard-update", () => {
      loadHistory();
    });
  }

  return {
    history,
    searchQuery,
    selectedIndex,
    activeFilter,
    previewItem,
    previewContent,
    filteredHistory,
    loadHistory,
    pasteItem,
    deleteItem,
    toggleSensitive,
    clearHistory,
    getImageSrc,
    scrollToSelected,
    setupClipboardListeners,
  };
}
