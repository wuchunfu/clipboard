export interface ClipboardItem {
  content: string;
  kind: string;
  timestamp: string;
  is_sensitive?: boolean;
}

export interface AppConfig {
  shortcut: string;
  max_history_size: number;
  language: string;
}
