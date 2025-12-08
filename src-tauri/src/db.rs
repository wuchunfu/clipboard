use crate::crypto::Crypto;
use crate::models::ClipboardItem;
use rusqlite::{params, Connection, OptionalExtension, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Database {
    conn: Mutex<Connection>,
    crypto: Arc<Crypto>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P, crypto: Arc<Crypto>) -> Result<Self> {
        let mut conn = Connection::open(path)?;

        let tx = conn.transaction()?;
        let version: i32 = tx.query_row("PRAGMA user_version", [], |row| row.get(0))?;

        if version < 1 {
            tx.execute(
                "CREATE TABLE IF NOT EXISTS history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    content TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    timestamp TEXT NOT NULL,
                    is_sensitive BOOLEAN NOT NULL DEFAULT 0
                )",
                [],
            )?;

            tx.execute(
                "CREATE INDEX IF NOT EXISTS idx_content_kind ON history (content, kind)",
                [],
            )?;

            tx.execute("PRAGMA user_version = 1", [])?;
        }

        tx.commit()?;

        Ok(Self {
            conn: Mutex::new(conn),
            crypto,
        })
    }

    pub fn get_history(&self, page: usize, page_size: usize) -> Result<Vec<ClipboardItem>> {
        let conn = self.conn.lock().unwrap();
        let offset = (page - 1) * page_size;
        let mut stmt = conn.prepare(
            "SELECT id, content, kind, timestamp, is_sensitive FROM history ORDER BY timestamp DESC LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(params![page_size, offset], |row| {
            let id: i64 = row.get(0)?;
            let content: String = row.get(1)?;
            let kind: String = row.get(2)?;
            let timestamp: String = row.get(3)?;
            let is_sensitive: bool = row.get(4)?;

            let mut final_content = if is_sensitive && kind == "text" {
                self.crypto.decrypt(&content).unwrap_or_else(|_| content)
            } else {
                content
            };

            // Truncate content for list view (performance optimization)
            if kind == "text" && final_content.chars().count() > 200 {
                final_content = final_content.chars().take(200).collect();
            }

            Ok(ClipboardItem {
                id: Some(id),
                content: final_content,
                kind,
                timestamp,
                is_sensitive,
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn insert_item(&self, item: &ClipboardItem, max_size: usize) -> Result<Vec<ClipboardItem>> {
        let conn = self.conn.lock().unwrap();
        let mut pruned_items = Vec::new();

        let content_to_store = if item.is_sensitive && item.kind == "text" {
            self.crypto
                .encrypt(&item.content)
                .unwrap_or(item.content.clone())
        } else {
            item.content.clone()
        };

        // Deduplicate: Update timestamp if exists (for non-sensitive items mostly)
        let updated_count = conn.execute(
            "UPDATE history SET timestamp = ?1 WHERE content = ?2 AND kind = ?3",
            params![item.timestamp, content_to_store, item.kind],
        )?;

        if updated_count == 0 {
            // Insert new item
            conn.execute(
                "INSERT INTO history (content, kind, timestamp, is_sensitive) VALUES (?1, ?2, ?3, ?4)",
                params![
                    content_to_store,
                    item.kind,
                    item.timestamp,
                    item.is_sensitive
                ],
            )?;
        }

        // Prune if exceeding max_size
        let count: usize = conn.query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))?;
        if count > max_size {
            let delete_count = count - max_size;

            // Fetch items to be deleted first (oldest timestamp)
            let mut stmt = conn.prepare(&format!(
                "SELECT content, kind, timestamp, is_sensitive FROM history ORDER BY timestamp ASC LIMIT {}",
                delete_count
            ))?;

            let rows = stmt.query_map([], |row| {
                let content: String = row.get(0)?;
                let kind: String = row.get(1)?;
                let timestamp: String = row.get(2)?;
                let is_sensitive: bool = row.get(3)?;

                // We don't need to decrypt pruned items if we are just deleting files (images).
                // Images are not encrypted in this scheme (only text).
                // But if we wanted to return them, we should decrypt.
                // For now, let's decrypt to be consistent.
                let final_content = if is_sensitive && kind == "text" {
                    self.crypto.decrypt(&content).unwrap_or(content)
                } else {
                    content
                };

                Ok(ClipboardItem {
                    id: None,
                    content: final_content,
                    kind,
                    timestamp,
                    is_sensitive,
                })
            })?;

            for row in rows {
                if let Ok(item) = row {
                    pruned_items.push(item);
                }
            }

            // Delete them
            conn.execute(
                &format!(
                    "DELETE FROM history WHERE id IN (SELECT id FROM history ORDER BY timestamp ASC LIMIT {})",
                    delete_count
                ),
                [],
            )?;
        }

        Ok(pruned_items)
    }

    pub fn delete_item(&self, index: usize) -> Result<Option<ClipboardItem>> {
        // Index is from the frontend, which sees the list in DESC order (latest first).
        // So index 0 is the latest item (highest ID).
        // We need to find the ID of the item at that offset.
        let conn = self.conn.lock().unwrap();

        // Get the ID and details of the item at the specified offset
        let item: Option<(i64, ClipboardItem)> = conn
            .query_row(
                "SELECT id, content, kind, timestamp, is_sensitive FROM history ORDER BY timestamp DESC LIMIT 1 OFFSET ?1",
                params![index],
                |row| {
                    let id: i64 = row.get(0)?;
                    let content: String = row.get(1)?;
                    let kind: String = row.get(2)?;
                    let timestamp: String = row.get(3)?;
                    let is_sensitive: bool = row.get(4)?;

                    let final_content = if is_sensitive && kind == "text" {
                        self.crypto.decrypt(&content).unwrap_or(content)
                    } else {
                        content
                    };

                    Ok((
                        id,
                        ClipboardItem {
                            id: Some(id),
                            content: final_content,
                            kind,
                            timestamp,
                            is_sensitive,
                        },
                    ))
                },
            )
            .optional()?;

        if let Some((id, item)) = item {
            conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    pub fn toggle_sensitive(&self, index: usize) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        // Get item at index
        let item: Option<(i64, String, bool, String)> = conn
            .query_row(
                "SELECT id, content, is_sensitive, kind FROM history ORDER BY timestamp DESC LIMIT 1 OFFSET ?1",
                params![index],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                    ))
                },
            )
            .optional()?;

        if let Some((id, content, is_sensitive, kind)) = item {
            let new_state = !is_sensitive;
            let new_content = if kind == "text" {
                if new_state {
                    // Encrypt
                    self.crypto.encrypt(&content).unwrap_or(content)
                } else {
                    // Decrypt
                    self.crypto.decrypt(&content).unwrap_or(content)
                }
            } else {
                content
            };

            conn.execute(
                "UPDATE history SET is_sensitive = ?1, content = ?2 WHERE id = ?3",
                params![new_state, new_content, id],
            )?;
            Ok(new_state)
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    }

    pub fn clear_history(&self) -> Result<Vec<ClipboardItem>> {
        let conn = self.conn.lock().unwrap();

        // Get all items
        let mut stmt =
            conn.prepare("SELECT id, content, kind, timestamp, is_sensitive FROM history")?;
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let content: String = row.get(1)?;
            let kind: String = row.get(2)?;
            let timestamp: String = row.get(3)?;
            let is_sensitive: bool = row.get(4)?;

            let final_content = if is_sensitive && kind == "text" {
                self.crypto.decrypt(&content).unwrap_or(content)
            } else {
                content
            };

            Ok(ClipboardItem {
                id: Some(id),
                content: final_content,
                kind,
                timestamp,
                is_sensitive,
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }

        conn.execute("DELETE FROM history", [])?;
        Ok(items)
    }

    pub fn get_item_content(&self, id: i64) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let (content, is_sensitive, kind): (String, bool, String) = conn.query_row(
            "SELECT content, is_sensitive, kind FROM history WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        if is_sensitive && kind == "text" {
            Ok(self.crypto.decrypt(&content).unwrap_or(content))
        } else {
            Ok(content)
        }
    }
}
