use rusqlite::{params, Connection};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordType { Goal, Decision, Change, OpenQuestion, Handoff }
impl RecordType { fn as_str(self) -> &'static str { match self { Self::Goal => "goal", Self::Decision => "decision", Self::Change => "change", Self::OpenQuestion => "open_question", Self::Handoff => "handoff" } } }

#[derive(Clone, Copy)] pub enum ProposalOutcome { Accept, Reject }
pub struct Task { pub id: i64 }
pub struct Record { pub id: i64, pub content: String, pub is_stale: bool }
pub struct Snapshot { pub goal: String, pub confirmed: Vec<Record>, pub rejected: Vec<Record> }
pub struct Heartbeat { pub schema_version: String, pub goal: String, pub recent_changes: Vec<String> }
pub struct BoardView { pub goal: String, pub pending: Vec<Record> }

pub trait Storage { fn connection(&self) -> &Connection; }
pub struct SqliteStorage { connection: Connection }
impl Storage for SqliteStorage { fn connection(&self) -> &Connection { &self.connection } }

pub struct Board { storage: SqliteStorage }
pub struct TaskAccessApi<'a> { engine: &'a Board }
pub trait TaskTransport { fn request_heartbeat(&self, api: &TaskAccessApi<'_>, task_id: i64, schema_version: &str, actor: &str) -> rusqlite::Result<Heartbeat>; }
pub struct LoopbackRelay;
impl LoopbackRelay { pub fn request_heartbeat(&self, api: &TaskAccessApi<'_>, task_id: i64, schema_version: &str, actor: &str) -> rusqlite::Result<Heartbeat> { api.what_just_happened(task_id, schema_version, actor) } }
impl<'a> TaskAccessApi<'a> { pub fn new(engine: &'a Board) -> Self { Self { engine } } pub fn what_just_happened(&self, task_id: i64, schema_version: &str, actor: &str) -> rusqlite::Result<Heartbeat> { self.engine.what_just_happened(task_id, schema_version, actor) } pub fn pending(&self, task_id: i64) -> rusqlite::Result<Vec<Record>> { self.engine.pending(task_id) } pub fn accept(&self, id: i64) -> rusqlite::Result<()> { self.engine.review(id, ProposalOutcome::Accept) } pub fn reject(&self, id: i64) -> rusqlite::Result<()> { self.engine.review(id, ProposalOutcome::Reject) } pub fn edit_and_accept(&self, id: i64, content: &str) -> rusqlite::Result<()> { self.engine.edit_and_accept(id, content) } }
impl TaskTransport for LoopbackRelay { fn request_heartbeat(&self, api: &TaskAccessApi<'_>, task_id: i64, schema_version: &str, actor: &str) -> rusqlite::Result<Heartbeat> { api.what_just_happened(task_id, schema_version, actor) } }
impl Board {
 pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> { let storage = SqliteStorage { connection: Connection::open(path)? }; let board = Self { storage }; board.migrate()?; Ok(board) }
 fn migrate(&self) -> rusqlite::Result<()> { self.storage.connection().execute_batch("CREATE TABLE IF NOT EXISTS tasks(id INTEGER PRIMARY KEY, title TEXT NOT NULL, goal TEXT NOT NULL); CREATE TABLE IF NOT EXISTS records(id INTEGER PRIMARY KEY, task_id INTEGER NOT NULL, kind TEXT NOT NULL, content TEXT NOT NULL, actor TEXT NOT NULL, status TEXT NOT NULL, is_stale INTEGER NOT NULL DEFAULT 0);") }
 pub fn create_task(&mut self, title: &str, goal: &str) -> rusqlite::Result<Task> { self.storage.connection().execute("INSERT INTO tasks(title, goal) VALUES (?1, ?2)", params![title, goal])?; Ok(Task { id: self.storage.connection().last_insert_rowid() }) }
 pub fn propose(&mut self, task_id: i64, kind: RecordType, content: &str, actor: &str) -> rusqlite::Result<Record> { let version = self.confirmed_version(task_id)?; self.propose_from_version(task_id, kind, content, actor, version) }
 pub fn propose_from_version(&mut self, task_id: i64, kind: RecordType, content: &str, actor: &str, base_version: i64) -> rusqlite::Result<Record> { let is_stale = base_version < self.confirmed_version(task_id)?; self.storage.connection().execute("INSERT INTO records(task_id, kind, content, actor, status, is_stale) VALUES (?1, ?2, ?3, ?4, 'proposed', ?5)", params![task_id, kind.as_str(), content, actor, is_stale])?; Ok(Record { id: self.storage.connection().last_insert_rowid(), content: content.into(), is_stale }) }
 fn confirmed_version(&self, task_id: i64) -> rusqlite::Result<i64> { self.storage.connection().query_row("SELECT 1 + COUNT(*) FROM records WHERE task_id=?1 AND status='confirmed'", params![task_id], |row| row.get(0)) }
 pub fn review(&self, id: i64, outcome: ProposalOutcome) -> rusqlite::Result<()> { let status = match outcome { ProposalOutcome::Accept => "confirmed", ProposalOutcome::Reject => "rejected" }; self.storage.connection().execute("UPDATE records SET status=?1 WHERE id=?2 AND status='proposed'", params![status, id])?; Ok(()) }
 pub fn edit_and_accept(&self, id: i64, content: &str) -> rusqlite::Result<()> { self.storage.connection().execute("UPDATE records SET content=?1, status='confirmed' WHERE id=?2 AND status='proposed'", params![content, id])?; Ok(()) }
 pub fn resume(&self, task_id: i64) -> rusqlite::Result<Snapshot> { let goal = self.storage.connection().query_row("SELECT goal FROM tasks WHERE id=?1", params![task_id], |r| r.get(0))?; let load = |status| -> rusqlite::Result<Vec<Record>> { let mut s = self.storage.connection().prepare("SELECT id, content, is_stale FROM records WHERE task_id=?1 AND status=?2 ORDER BY id")?; let rows = s.query_map(params![task_id, status], |r| Ok(Record { id: r.get(0)?, content: r.get(1)?, is_stale: r.get(2)? }))?; let records: rusqlite::Result<Vec<Record>> = rows.collect(); records }; Ok(Snapshot { goal, confirmed: load("confirmed")?, rejected: load("rejected")? }) }
 pub fn what_just_happened(&self, task_id: i64, schema_version: &str, _actor: &str) -> rusqlite::Result<Heartbeat> { let snapshot = self.resume(task_id)?; Ok(Heartbeat { schema_version: schema_version.into(), goal: snapshot.goal, recent_changes: snapshot.confirmed.into_iter().map(|record| record.content).collect() }) }
 pub fn pending(&self, task_id: i64) -> rusqlite::Result<Vec<Record>> { let mut statement = self.storage.connection().prepare("SELECT id, content, is_stale FROM records WHERE task_id=?1 AND status='proposed' ORDER BY id")?; let rows = statement.query_map(params![task_id], |row| Ok(Record { id: row.get(0)?, content: row.get(1)?, is_stale: row.get(2)? }))?; rows.collect() }
}
impl BoardView { pub fn load(api: &TaskAccessApi<'_>, task_id: i64) -> rusqlite::Result<Self> { let heartbeat = api.what_just_happened(task_id, "v1", "board")?; Ok(Self { goal: heartbeat.goal, pending: api.pending(task_id)? }) } pub fn accept(&self, api: &TaskAccessApi<'_>, proposal_id: i64) -> rusqlite::Result<()> { api.accept(proposal_id) } }
