use synclined_core::{Board, RecordType, SensitiveReleaseState};
use rusqlite::Connection;

#[test]
fn explicit_sensitive_markers_require_release_review_case_insensitively() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine.create_task("Fix login timeout", "Keep session behavior unchanged").unwrap();

    for content in ["api key: abc", "PASSWORD=abc", "a Secret value", "ACCESS TOKEN: abc"] {
        let proposal = engine.propose(task.id, RecordType::Change, content, "cloud-code").unwrap();
        assert_eq!(proposal.sensitive_release_state, SensitiveReleaseState::Pending);
    }
}

#[test]
fn routine_content_does_not_require_release_review_and_state_persists() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine.create_task("Fix login timeout", "Keep session behavior unchanged").unwrap();
    let proposal = engine.propose(task.id, RecordType::OpenQuestion, "Should we measure failures?", "cloud-code").unwrap();

    assert_eq!(proposal.sensitive_release_state, SensitiveReleaseState::NotRequired);
    drop(engine);

    let reopened = Board::open(database.path()).unwrap();
    assert_eq!(reopened.pending(task.id).unwrap()[0].sensitive_release_state, SensitiveReleaseState::NotRequired);
}

#[test]
fn pending_sensitive_release_state_persists_after_reopening() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine.create_task("Fix login timeout", "Keep session behavior unchanged").unwrap();
    engine.propose(task.id, RecordType::Change, "secret: abc", "cloud-code").unwrap();
    drop(engine);

    let reopened = Board::open(database.path()).unwrap();
    assert_eq!(reopened.pending(task.id).unwrap()[0].sensitive_release_state, SensitiveReleaseState::Pending);
}

#[test]
fn unrecognized_persisted_sensitive_release_state_fails_closed() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine.create_task("Fix login timeout", "Keep session behavior unchanged").unwrap();
    let proposal = engine.propose(task.id, RecordType::Change, "Routine change", "cloud-code").unwrap();
    drop(engine);

    let connection = Connection::open(database.path()).unwrap();
    connection.execute("UPDATE records SET sensitive_release_state='unexpected' WHERE id=?1", [proposal.id]).unwrap();
    drop(connection);

    let reopened = Board::open(database.path()).unwrap();
    assert_eq!(reopened.pending(task.id).unwrap()[0].sensitive_release_state, SensitiveReleaseState::Pending);
}
