use rusqlite::{backup::Backup, types::Value, Connection, OpenFlags};
use spin_sdk::http::{Params, Request, Response, Router};
use spin_sdk::http_component;
use std::sync::OnceLock;

struct Db(Connection);
unsafe impl Send for Db {}
unsafe impl Sync for Db {}

static DB: OnceLock<Db> = OnceLock::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let mut db_path = String::new();
    std::io::stdin()
        .read_line(&mut db_path)
        .expect("failed to read db path from stdin");
    let db_path = db_path.trim();
    assert!(!db_path.is_empty(), "usage: echo <db-path> | wizer ...");

    eprintln!("Loading {db_path} into memory");

    let src = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("failed to open database");
    let mut dst = Connection::open_in_memory().expect("failed to open in-memory db");

    let backup = Backup::new(&src, &mut dst).expect("failed to init backup");
    backup.step(-1).expect("backup failed");
    drop(backup);
    drop(src);

    DB.set(Db(dst)).ok().expect("DB already initialized");
}

#[http_component]
fn handle(req: Request) -> Response {
    let mut router = Router::suffix();
    router.get("/tables", get_tables);
    router.post("/query", post_query);

    router.handle(req)
}

fn get_tables(_req: Request, _params: Params) -> anyhow::Result<Response> {
    let sql = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";
    query(sql)
}

fn post_query(req: Request, _params: Params) -> anyhow::Result<Response> {
    let sql = String::from_utf8_lossy(req.body()).trim().to_string();
    if sql.is_empty() {
        return json_response(400, r#"{"error":"missing SQL in request body"}"#);
    }
    query(&sql)
}

fn query(sql: &str) -> anyhow::Result<Response> {
    let db = &DB.get().expect("DB not initialized").0;
    let mut stmt = db.prepare(sql)?;
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
