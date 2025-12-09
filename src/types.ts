export interface ClipboardItem {
  id?: number;
  content: string;
  kind: string;
  timestamp: string;
  is_sensitive?: boolean;
  is_pinned?: boolean;
}

export interface AppConfig {
  shortcut: string;
  max_history_size: number;
  language: string;
  theme: string;
  sensitive_apps: string[];
}
