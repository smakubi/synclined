use synclined_core::{Board, LoopbackRelay, ProposalOutcome, RecordType, TaskAccessApi};

#[test]
fn revoked_actor_is_immediately_denied_reads_subscriptions_and_both_proposal_paths() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();

    let api = TaskAccessApi::new(&engine);
    api.revoke_session("revoked-agent").unwrap();
    assert!(api
        .what_just_happened(task.id, "v1", "revoked-agent")
        .is_err());
    assert!(LoopbackRelay
        .request_heartbeat(&api, task.id, "v1", "revoked-agent")
        .is_err());
    assert!(api.subscribe("revoked-agent").is_err());

    assert!(engine
        .propose(
            task.id,
            RecordType::Change,
            "Routine change",
            "revoked-agent"
        )
        .is_err());
    assert!(engine
        .propose_from_version(
            task.id,
            RecordType::Change,
            "Routine change",
            "revoked-agent",
            1
        )
        .is_err());
}

#[test]
fn revocation_persists_while_developer_history_remains_available() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();
    let record = engine
        .propose(
            task.id,
            RecordType::Change,
            "Increase timeout",
            "revoked-agent",
        )
        .unwrap();
    engine.review(record.id, ProposalOutcome::Accept).unwrap();
    engine.revoke_session("revoked-agent").unwrap();
    drop(engine);

    let reopened = Board::open(database.path()).unwrap();
    assert_eq!(
        reopened.resume(task.id).unwrap().confirmed[0].content,
        "Increase timeout"
    );
    assert!(TaskAccessApi::new(&reopened)
        .what_just_happened(task.id, "v1", "revoked-agent")
        .is_err());
}

#[test]
fn revoking_one_actor_does_not_block_another_actor() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();
    engine.revoke_session("revoked-agent").unwrap();

    let proposal = engine
        .propose(
            task.id,
            RecordType::Change,
            "Increase timeout",
            "active-agent",
        )
        .unwrap();
    engine.review(proposal.id, ProposalOutcome::Accept).unwrap();
    let api = TaskAccessApi::new(&engine);
    assert!(api.subscribe("active-agent").is_ok());
    assert_eq!(
        api.what_just_happened(task.id, "v1", "active-agent")
            .unwrap()
            .recent_changes,
        vec!["Increase timeout"]
    );
}
