use rusqlite::{params, Connection};
use synclined_core::{Board, ProposalOutcome, RecordType, TaskAccessApi};

fn confirm(board: &mut Board, task_id: i64, kind: RecordType, content: &str) -> i64 {
    let record = board.propose(task_id, kind, content, "active-agent").unwrap();
    board.review(record.id, ProposalOutcome::Accept).unwrap();
    record.id
}

#[test]
fn handoff_maps_confirmed_allowlisted_records_in_order_and_excludes_everything_else() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut board = Board::open(database.path()).unwrap();
    let task = board.create_task("Login timeout", "Keep logins reliable").unwrap();

    confirm(&mut board, task.id, RecordType::Decision, "Use a retry budget");
    confirm(&mut board, task.id, RecordType::OpenQuestion, "Which timeout is safe?");
    confirm(&mut board, task.id, RecordType::Change, "Add timeout metrics");
    confirm(&mut board, task.id, RecordType::Decision, "Keep retries bounded");
    confirm(&mut board, task.id, RecordType::Goal, "Ignore this record goal");
    confirm(&mut board, task.id, RecordType::Handoff, "Ignore this record handoff");
    let _proposed = board.propose(task.id, RecordType::Decision, "Do not include proposed", "active-agent").unwrap();
    let rejected = board.propose(task.id, RecordType::Change, "Do not include rejected", "active-agent").unwrap();
    board.review(rejected.id, ProposalOutcome::Reject).unwrap();

    Connection::open(database.path()).unwrap().execute(
        "INSERT INTO records(task_id, kind, content, actor, status, sensitive_release_state) VALUES (?1, 'chat', 'Do not include chat', 'active-agent', 'confirmed', 'not_required')",
        params![task.id],
    ).unwrap();

    let api = TaskAccessApi::new(&board);
    let handoff = api.compose_handoff(task.id, "v1", "active-agent").unwrap();
    assert_eq!(handoff.schema_version, "v1");
    assert_eq!(handoff.goal, "Keep logins reliable");
    assert_eq!(handoff.decisions, vec!["Use a retry budget", "Keep retries bounded"]);
    assert_eq!(handoff.open_questions, vec!["Which timeout is safe?"]);
    assert_eq!(handoff.next_steps, vec!["Add timeout metrics"]);
}

#[test]
fn handoff_excludes_pending_and_unknown_sensitive_records_until_explicit_approval() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut board = Board::open(database.path()).unwrap();
    let task = board.create_task("Login timeout", "Keep logins reliable").unwrap();
    let pending = confirm(&mut board, task.id, RecordType::Decision, "Store API KEY securely");
    let unknown = confirm(&mut board, task.id, RecordType::Change, "Unknown state must stay hidden");
    confirm(&mut board, task.id, RecordType::Change, "Publish routine metric");
    Connection::open(database.path()).unwrap().execute(
        "UPDATE records SET sensitive_release_state='unexpected' WHERE id=?1",
        params![unknown],
    ).unwrap();

    let api = TaskAccessApi::new(&board);
    let before_approval = api.compose_handoff(task.id, "v1", "active-agent").unwrap();
    assert!(before_approval.decisions.is_empty());
    assert_eq!(before_approval.next_steps, vec!["Publish routine metric"]);
    api.approve_sensitive_release(pending).unwrap();
    let after_approval = api.compose_handoff(task.id, "v1", "active-agent").unwrap();
    assert_eq!(after_approval.decisions, vec!["Store API KEY securely"]);
    assert_eq!(after_approval.next_steps, vec!["Publish routine metric"]);
}

#[test]
fn revoked_actors_cannot_compose_handoffs_while_active_actors_can() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut board = Board::open(database.path()).unwrap();
    let task = board.create_task("Login timeout", "Keep logins reliable").unwrap();
    confirm(&mut board, task.id, RecordType::Change, "Publish routine metric");
    let api = TaskAccessApi::new(&board);
    api.revoke_session("revoked-agent").unwrap();

    assert!(api.compose_handoff(task.id, "v1", "revoked-agent").is_err());
    assert_eq!(api.compose_handoff(task.id, "v1", "active-agent").unwrap().next_steps, vec!["Publish routine metric"]);
}
