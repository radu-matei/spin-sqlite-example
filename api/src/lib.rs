use rusqlite::{types::Value, Connection, OpenFlags};
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;

const DB_URI: &str = "file:/chinook.db?immutable=1";

fn open_db() -> anyhow::Result<Connection> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI;
    Ok(Connection::open_with_flags(DB_URI, flags)?)
}

#[http_component]
fn handle(req: Request) -> anyhow::Result<impl IntoResponse> {
    let path_info = req
        .header("spin-path-info")
        .map(|v| String::from_utf8_lossy(v.as_bytes()).into_owned())
        .unwrap_or_else(|| "/".into());
    let path = path_info.trim_end_matches('/');
    let path = if path.is_empty() { "/" } else { path };

    let sql = match path {
        "/tables" => {
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name".to_string()
        }
        "/query" => {
            let body = String::from_utf8_lossy(req.body()).trim().to_string();
            if body.is_empty() {
                return json_response(400, r#"{"error":"missing SQL in request body"}"#);
            }
            body
        }
        _ => return json_response(404, r#"{"error":"not found"}"#),
    };

    let db = open_db()?;
    let mut stmt = db.prepare(&sql)?;
    let columns: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();
    let rows: Vec<serde_json::Map<String, serde_json::Value>> = stmt
        .query_map([], |row| {
            Ok(columns
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    let val: Value = row.get_unwrap(i);
                    (col.clone(), value_to_json(&val))
                })
                .collect())
        })?
        .filter_map(|r| r.ok())
        .collect();

    json_response(200, &serde_json::to_string(&rows)?)
}

fn value_to_json(val: &Value) -> serde_json::Value {
    match val {
        Value::Null => serde_json::Value::Null,
        Value::Integer(n) => serde_json::json!(*n),
        Value::Real(f) => serde_json::json!(*f),
        Value::Text(s) => serde_json::Value::String(s.clone()),
        Value::Blob(b) => serde_json::json!(format!("<{} bytes>", b.len())),
    }
}

fn json_response(status: u16, body: &str) -> anyhow::Result<Response> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(body.to_owned())
        .build())
}
