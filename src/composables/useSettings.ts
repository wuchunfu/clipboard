import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { useI18n } from "vue-i18n";
import { useToast } from "./useToast";
import type { AppConfig } from "../types";

export function useSettings() {
  const { t, locale } = useI18n();
  const { showToast } = useToast();

  const config = ref<AppConfig>({
    shortcut: "CommandOrControl+Shift+V",
    max_history_size: 20,
    language: "auto",
    theme: "auto",
    sensitive_apps: [],
    compact_mode: false,
  });

  const showSettings = ref(false);
  const tempShortcut = ref("");
  const tempMaxSize = ref(20);
  const tempLanguage = ref("auto");
  const tempTheme = ref("auto");
  const tempSensitiveApps = ref<string[]>([]);
  const tempCompactMode = ref(false);
  const isRecording = ref(false);
  const isPaused = ref(false);
  const isAutoStart = ref(false);

  // Theme handling
  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

  function applyTheme(theme: string) {
    const isDark = theme === "dark" || (theme === "auto" && mediaQuery.matches);

    if (isDark) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }

  // Listen for system theme changes
  mediaQuery.addEventListener("change", () => {
    if (config.value.theme === "auto") {
      applyTheme("auto");
    }
  });

  async function loadConfig() {
    try {
      config.value = await invoke<AppConfig>("get_config");
      tempShortcut.value = config.value.shortcut;
      tempMaxSize.value = config.value.max_history_size;
      tempLanguage.value = config.value.language || "auto";
      tempTheme.value = config.value.theme || "auto";
      tempSensitiveApps.value = [...(config.value.sensitive_apps || [])];
      tempCompactMode.value = config.value.compact_mode || false;

      // Apply language
      if (config.value.language === "auto") {
        locale.value = navigator.language.startsWith("zh") ? "zh" : "en";
      } else {
        locale.value = config.value.language;
      }

      // Apply theme
      applyTheme(config.value.theme || "auto");

      // Load paused state
      isPaused.value = await invoke<boolean>("get_paused");
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  }

  async function saveConfig() {
    try {
      await invoke("save_config", {
        shortcut: tempShortcut.value,
        maxHistorySize: tempMaxSize.value,
        language: tempLanguage.value,
        theme: tempTheme.value,
        sensitiveApps: tempSensitiveApps.value,
        compactMode: tempCompactMode.value,
      });
      await loadConfig();
      showSettings.value = false;
      showToast(t("toast.settingsSaved"));
    } catch (e) {
      console.error("Failed to save config:", e);
      alert(t("toast.settingsSaveError") + e);
    }
  }

  function openSettings() {
    showSettings.value = true;
    tempShortcut.value = config.value.shortcut;
    tempMaxSize.value = config.value.max_history_size;
    tempLanguage.value = config.value.language || "auto";
    tempTheme.value = config.value.theme || "auto";
    tempSensitiveApps.value = [...(config.value.sensitive_apps || [])];
    tempCompactMode.value = config.value.compact_mode || false;
    isEnabled().then((enabled) => {
      isAutoStart.value = enabled;
    });
  }

  async function toggleAutoStart() {
    try {
      if (isAutoStart.value) {
        await disable();
        isAutoStart.value = false;
        showToast(t("toast.autoStartDisabled"));
      } else {
        await enable();
        isAutoStart.value = true;
        showToast(t("toast.autoStartEnabled"));
      }
    } catch (e) {
      console.error("Failed to toggle autostart:", e);
    }
  }

  async function togglePause() {
    try {
      const newState = !isPaused.value;
      await invoke("set_paused", { paused: newState });
      isPaused.value = newState;
      showToast(
        newState ? t("toast.recordingPaused") : t("toast.recordingResumed")
      );
    } catch (e) {
      console.error("Failed to toggle pause:", e);
    }
  }

  function startRecording(e: MouseEvent) {
    isRecording.value = true;
    tempShortcut.value = t("settings.recordShortcut");
    (e.target as HTMLInputElement).focus();
  }

  function handleShortcutKeydown(e: KeyboardEvent) {
    if (!isRecording.value) return;
    e.preventDefault();
    e.stopPropagation();

    const modifiers = [];
    if (e.metaKey) modifiers.push("CommandOrControl");
    if (e.ctrlKey) modifiers.push("Control");
    if (e.altKey) modifiers.push("Alt");
    if (e.shiftKey) modifiers.push("Shift");

    let key = e.key.toUpperCase();

    const keyMap: Record<string, string> = {
      " ": "Space",
      ARROWUP: "Up",
      ARROWDOWN: "Down",
      ARROWLEFT: "Left",
      ARROWRIGHT: "Right",
      ENTER: "Return",
      ESCAPE: "Escape",
      BACKSPACE: "Backspace",
      TAB: "Tab",
    };

    if (keyMap[key]) {
      key = keyMap[key];
    }

    if (["META", "CONTROL", "ALT", "SHIFT"].includes(key)) {
      return;
    }

    const shortcut = [...modifiers, key].join("+");
    tempShortcut.value = shortcut;
    isRecording.value = false;
  }

  async function setupConfigListeners() {
    await listen("config-updated", () => {
      loadConfig();
    });
    await listen("open-settings", () => {
      openSettings();
    });
    await listen("pause-state-changed", (event) => {
      isPaused.value = event.payload as boolean;
    });
  }

  return {
    config,
    showSettings,
    tempShortcut,
    tempMaxSize,
    tempLanguage,
    tempTheme,
    tempSensitiveApps,
    tempCompactMode,
    isRecording,
    isPaused,
    isAutoStart,
    loadConfig,
    saveConfig,
    openSettings,
    toggleAutoStart,
    togglePause,
    startRecording,
    handleShortcutKeydown,
    setupConfigListeners,
  };
}
