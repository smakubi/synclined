use rusqlite::{params, Connection};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordType { Goal, Decision, Change, OpenQuestion, Handoff }
impl RecordType { fn as_str(self) -> &'static str { match self { Self::Goal => "goal", Self::Decision => "decision", Self::Change => "change", Self::OpenQuestion => "open_question", Self::Handoff => "handoff" } } }

#[derive(Clone, Copy)] pub enum ProposalOutcome { Accept, Reject }
pub struct Task { pub id: i64 }
pub struct Record { pub id: i64, pub content: String }
pub struct Snapshot { pub goal: String, pub confirmed: Vec<Record>, pub rejected: Vec<Record> }
pub struct Heartbeat { pub schema_version: String, pub goal: String, pub recent_changes: Vec<String> }

pub trait Storage { fn connection(&self) -> &Connection; }
pub struct SqliteStorage { connection: Connection }
impl Storage for SqliteStorage { fn connection(&self) -> &Connection { &self.connection } }

pub struct Board { storage: SqliteStorage }
impl Board {
 pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> { let storage = SqliteStorage { connection: Connection::open(path)? }; let board = Self { storage }; board.migrate()?; Ok(board) }
 fn migrate(&self) -> rusqlite::Result<()> { self.storage.connection().execute_batch("CREATE TABLE IF NOT EXISTS tasks(id INTEGER PRIMARY KEY, title TEXT NOT NULL, goal TEXT NOT NULL); CREATE TABLE IF NOT EXISTS records(id INTEGER PRIMARY KEY, task_id INTEGER NOT NULL, kind TEXT NOT NULL, content TEXT NOT NULL, actor TEXT NOT NULL, status TEXT NOT NULL);") }
 pub fn create_task(&mut self, title: &str, goal: &str) -> rusqlite::Result<Task> { self.storage.connection().execute("INSERT INTO tasks(title, goal) VALUES (?1, ?2)", params![title, goal])?; Ok(Task { id: self.storage.connection().last_insert_rowid() }) }
 pub fn propose(&mut self, task_id: i64, kind: RecordType, content: &str, actor: &str) -> rusqlite::Result<Record> { self.storage.connection().execute("INSERT INTO records(task_id, kind, content, actor, status) VALUES (?1, ?2, ?3, ?4, 'proposed')", params![task_id, kind.as_str(), content, actor])?; Ok(Record { id: self.storage.connection().last_insert_rowid(), content: content.into() }) }
 pub fn review(&mut self, id: i64, outcome: ProposalOutcome) -> rusqlite::Result<()> { let status = match outcome { ProposalOutcome::Accept => "confirmed", ProposalOutcome::Reject => "rejected" }; self.storage.connection().execute("UPDATE records SET status=?1 WHERE id=?2 AND status='proposed'", params![status, id])?; Ok(()) }
 pub fn resume(&self, task_id: i64) -> rusqlite::Result<Snapshot> { let goal = self.storage.connection().query_row("SELECT goal FROM tasks WHERE id=?1", params![task_id], |r| r.get(0))?; let load = |status| -> rusqlite::Result<Vec<Record>> { let mut s = self.storage.connection().prepare("SELECT id, content FROM records WHERE task_id=?1 AND status=?2 ORDER BY id")?; let rows = s.query_map(params![task_id, status], |r| Ok(Record { id: r.get(0)?, content: r.get(1)? }))?; let records: rusqlite::Result<Vec<Record>> = rows.collect(); records }; Ok(Snapshot { goal, confirmed: load("confirmed")?, rejected: load("rejected")? }) }
 pub fn what_just_happened(&self, task_id: i64, schema_version: &str, _actor: &str) -> rusqlite::Result<Heartbeat> { let snapshot = self.resume(task_id)?; Ok(Heartbeat { schema_version: schema_version.into(), goal: snapshot.goal, recent_changes: snapshot.confirmed.into_iter().map(|record| record.content).collect() }) }
}
