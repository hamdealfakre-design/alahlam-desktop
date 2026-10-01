#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::{params, Connection, NO_PARAMS};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

fn db_path() -> PathBuf {
  let mut p = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
  p.push("data");
  fs::create_dir_all(&p).ok();
  p.push("alahlam.db");
  p
}

fn get_conn() -> Result<Connection, String> {
  let p = db_path();
  Connection::open(p).map_err(|e| e.to_string())
}

#[tauri::command]
fn init_db() -> Result<(), String> {
  let conn = get_conn()?;
  conn.execute_batch(
    r#"
    PRAGMA foreign_keys = ON;
    CREATE TABLE IF NOT EXISTS rooms (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      number TEXT UNIQUE,
      rtype TEXT,
      rate INTEGER,
      status TEXT
    );
    CREATE TABLE IF NOT EXISTS customers (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT,
      phone TEXT,
      identity TEXT
    );
    CREATE TABLE IF NOT EXISTS bookings (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      customer_id INTEGER,
      room_number TEXT,
      checkin TEXT,
      checkout TEXT,
      total INTEGER,
      status TEXT,
      created_at TEXT DEFAULT CURRENT_TIMESTAMP
    );
    CREATE TABLE IF NOT EXISTS accounts (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      code TEXT,
      name TEXT,
      type TEXT,
      parent_id INTEGER,
      balance INTEGER DEFAULT 0
    );
    CREATE TABLE IF NOT EXISTS journal_entries (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      date TEXT,
      description TEXT,
      created_at TEXT DEFAULT CURRENT_TIMESTAMP
    );
    CREATE TABLE IF NOT EXISTS journal_lines (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      entry_id INTEGER,
      account_id INTEGER,
      debit INTEGER DEFAULT 0,
      credit INTEGER DEFAULT 0
    );
    "#,
  ).map_err(|e| e.to_string())?;

  let cnt: i64 = conn.query_row("SELECT COUNT(*) FROM rooms", NO_PARAMS, |r| r.get(0)).unwrap_or(0);
  if cnt == 0 {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO rooms (number, rtype, rate, status) VALUES (?1,?2,?3,?4)",
      params!["101","Single",200,"متاح"]).map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO rooms (number, rtype, rate, status) VALUES (?1,?2,?3,?4)",
      params!["102","Double",350,"محجوز"]).map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO rooms (number, rtype, rate, status) VALUES (?1,?2,?3,?4)",
      params!["103","Double",350,"متاح"]).map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO rooms (number, rtype, rate, status) VALUES (?1,?2,?3,?4)",
      params!["104","Suite",700,"صيانة"]).map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO accounts (code,name,type,balance) VALUES ('101','النقدية','أصل',250000)", NO_PARAMS).ok();
    tx.execute("INSERT INTO accounts (code,name,type,balance) VALUES ('401','إيرادات الحجوزات','إيراد',7200000)", NO_PARAMS).ok();
    tx.execute("INSERT INTO accounts (code,name,type,balance) VALUES ('501','مصاريف الصيانة','مصروف',980000)", NO_PARAMS).ok();
    tx.commit().map_err(|e| e.to_string())?;
  }

  Ok(())
}

#[tauri::command]
fn get_dashboard() -> Result<serde_json::Value, String> {
  let conn = get_conn()?;
  let revenue: i64 = conn.query_row("SELECT IFNULL(SUM(balance),0) FROM accounts WHERE type = 'إيراد'", NO_PARAMS, |r| r.get(0)).unwrap_or(0);
  let expenses: i64 = conn.query_row("SELECT IFNULL(SUM(balance),0) FROM accounts WHERE type = 'مصروف'", NO_PARAMS, |r| r.get(0)).unwrap_or(0);
  let rooms_total: i64 = conn.query_row("SELECT COUNT(*) FROM rooms", NO_PARAMS, |r| r.get(0)).unwrap_or(0);
  let rooms_booked: i64 = conn.query_row("SELECT COUNT(*) FROM rooms WHERE status = 'محجوز'", NO_PARAMS, |r| r.get(0)).unwrap_or(0);
  let pct = if rooms_total > 0 { format!("{}%", (rooms_booked * 100 / rooms_total)) } else { "0%".to_string() };
  let d = json!({"revenue": revenue, "expenses": expenses, "rooms_total": rooms_total, "rooms_booked": rooms_booked, "occupancy_pct": pct});
  Ok(d)
}

#[tauri::command]
fn list_rooms() -> Result<serde_json::Value, String> {
  let conn = get_conn()?;
  let mut stmt = conn.prepare("SELECT number, rtype, rate, status FROM rooms ORDER BY number").map_err(|e| e.to_string())?;
  let rows = stmt.query_map([], |r| {
    Ok(json!({
      "number": r.get::<_, String>(0)?,
      "rtype": r.get::<_, Option<String>>(1).unwrap_or_default(),
      "rate": r.get::<_, Option<i64>>(2).unwrap_or(0),
      "status": r.get::<_, Option<String>>(3).unwrap_or("متاح".to_string())
    }))
  }).map_err(|e| e.to_string())?;
  let mut arr = Vec::new();
  for r in rows { arr.push(r.map_err(|e| e.to_string())?); }
  Ok(serde_json::Value::Array(arr))
}

#[tauri::command]
fn list_bookings() -> Result<serde_json::Value, String> {
  let conn = get_conn()?;
  let mut stmt = conn.prepare("SELECT id, room_number, checkin, checkout, total, status, created_at FROM bookings ORDER BY created_at DESC LIMIT 50").map_err(|e| e.to_string())?;
  let rows = stmt.query_map([], |r| {
    Ok(json!({
      "id": r.get::<_, i64>(0)?,
      "room_number": r.get::<_, Option<String>>(1).unwrap_or_default(),
      "checkin": r.get::<_, Option<String>>(2).unwrap_or_default(),
      "checkout": r.get::<_, Option<String>>(3).unwrap_or_default(),
      "total": r.get::<_, Option<i64>>(4).unwrap_or(0),
      "status": r.get::<_, Option<String>>(5).unwrap_or("confirmed".to_string()),
      "created_at": r.get::<_, Option<String>>(6).unwrap_or_default()
    }))
  }).map_err(|e| e.to_string())?;
  let mut arr = Vec::new();
  for r in rows { arr.push(r.map_err(|e| e.to_string())?); }
  Ok(serde_json::Value::Array(arr))
}

#[derive(serde::Deserialize)]
struct BookingPayload { guest: Option<String>, room_number: String, checkin: String, checkout: String, total: i64 }

#[tauri::command]
fn create_booking(payload: BookingPayload) -> Result<serde_json::Value, String> {
  let conn = get_conn()?;
  let tx = conn.transaction().map_err(|e| e.to_string())?;
  let cust_id = if let Some(name) = payload.guest.clone() {
    tx.execute("INSERT INTO customers (name) VALUES (?1)", params![name]).map_err(|e| e.to_string())?;
    tx.last_insert_rowid()
  } else { 0 };
  tx.execute("INSERT INTO bookings (customer_id, room_number, checkin, checkout, total, status) VALUES (?1,?2,?3,?4,?5,?6)", params![if cust_id==0 { None::<i64> } else { Some(cust_id) }, payload.room_number, payload.checkin, payload.checkout, payload.total, "مسجل"]).map_err(|e| e.to_string())?;
  tx.execute("UPDATE rooms SET status = 'محجوز' WHERE number = ?1", params![payload.room_number]).ok();
  tx.commit().map_err(|e| e.to_string())?;
  Ok(json!({"ok": true}).into())
}

#[tauri::command]
fn create_journal_entry(payload: serde_json::Value) -> Result<serde_json::Value, String> {
  let conn = get_conn()?;
  let tx = conn.transaction().map_err(|e| e.to_string())?;
  let date = payload.get("date").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let desc = payload.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
  tx.execute("INSERT INTO journal_entries (date, description) VALUES (?1,?2)", params![date, desc]).map_err(|e| e.to_string())?;
  let entry_id = tx.last_insert_rowid();
  if let Some(lines) = payload.get("lines").and_then(|v| v.as_array()) {
    for l in lines {
      let acc = l.get("account_id").and_then(|v| v.as_i64()).unwrap_or(0);
      let debit = l.get("debit").and_then(|v| v.as_i64()).unwrap_or(0);
      let credit = l.get("credit").and_then(|v| v.as_i64()).unwrap_or(0);
      tx.execute("INSERT INTO journal_lines (entry_id, account_id, debit, credit) VALUES (?1,?2,?3,?4)", params![entry_id, acc, debit, credit]).map_err(|e| e.to_string())?;
    }
  }
  tx.commit().map_err(|e| e.to_string())?;
  Ok(json!({"ok": true}).into())
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![ init_db, get_dashboard, list_rooms, list_bookings, create_booking, create_journal_entry ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
