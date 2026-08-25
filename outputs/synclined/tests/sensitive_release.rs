use rusqlite::Connection;
use synclined_core::{Board, LoopbackRelay, ProposalOutcome, RecordType, TaskAccessApi};

#[test]
fn relay_excludes_pending_sensitive_records_until_explicit_approval() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();

    let routine = engine
        .propose(
            task.id,
            RecordType::Change,
            "Increase timeout after evidence",
            "cloud-code",
        )
        .unwrap();
    engine.review(routine.id, ProposalOutcome::Accept).unwrap();
    let sensitive = engine
        .propose(task.id, RecordType::Change, "secret: abc", "cloud-code")
        .unwrap();
    engine
        .review(sensitive.id, ProposalOutcome::Accept)
        .unwrap();

    let api = TaskAccessApi::new(&engine);
    let relay = LoopbackRelay;
    let before = relay
        .request_heartbeat(&api, task.id, "v1", "phone-client")
        .unwrap();
    assert_eq!(
        before.recent_changes,
        vec!["Increase timeout after evidence"]
    );

    api.approve_sensitive_release(sensitive.id).unwrap();
    let after = relay
        .request_heartbeat(&api, task.id, "v1", "phone-client")
        .unwrap();
    assert_eq!(
        after.recent_changes,
        vec!["Increase timeout after evidence", "secret: abc"]
    );
}

#[test]
fn sensitive_release_approval_persists_after_reopening() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();
    let sensitive = engine
        .propose(
            task.id,
            RecordType::Change,
            "access token: abc",
            "cloud-code",
        )
        .unwrap();
    engine
        .review(sensitive.id, ProposalOutcome::Accept)
        .unwrap();
    TaskAccessApi::new(&engine)
        .approve_sensitive_release(sensitive.id)
        .unwrap();
    drop(engine);

    let reopened = Board::open(database.path()).unwrap();
    let heartbeat = LoopbackRelay
        .request_heartbeat(
            &TaskAccessApi::new(&reopened),
            task.id,
            "v1",
            "phone-client",
        )
        .unwrap();
    assert_eq!(heartbeat.recent_changes, vec!["access token: abc"]);
}

#[test]
fn unknown_persisted_sensitive_state_is_excluded_from_relay_output() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();
    let sensitive = engine
        .propose(task.id, RecordType::Change, "password: abc", "cloud-code")
        .unwrap();
    engine
        .review(sensitive.id, ProposalOutcome::Accept)
        .unwrap();
    drop(engine);

    let connection = Connection::open(database.path()).unwrap();
    connection
        .execute(
            "UPDATE records SET sensitive_release_state='unexpected' WHERE id=?1",
            [sensitive.id],
        )
        .unwrap();
    drop(connection);

    let reopened = Board::open(database.path()).unwrap();
    let heartbeat = LoopbackRelay
        .request_heartbeat(
            &TaskAccessApi::new(&reopened),
            task.id,
            "v1",
            "phone-client",
        )
        .unwrap();
    assert!(heartbeat.recent_changes.is_empty());
}

#[test]
fn editing_a_routine_proposal_to_sensitive_content_requires_approval_before_release() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();
    let proposal = engine
        .propose(
            task.id,
            RecordType::Change,
            "Increase timeout",
            "cloud-code",
        )
        .unwrap();

    engine.edit_and_accept(proposal.id, "secret: abc").unwrap();
    let api = TaskAccessApi::new(&engine);
    let relay = LoopbackRelay;
    let before = relay
        .request_heartbeat(&api, task.id, "v1", "phone-client")
        .unwrap();
    assert!(before.recent_changes.is_empty());

    api.approve_sensitive_release(proposal.id).unwrap();
    let after = relay
        .request_heartbeat(&api, task.id, "v1", "phone-client")
        .unwrap();
    assert_eq!(after.recent_changes, vec!["secret: abc"]);
}
