use synclined_core::{Board, LoopbackRelay, RecordType, TaskAccessApi};

#[test]
fn reviewed_handoff_survives_reopen_and_requires_sensitive_release_approval() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut board = Board::open(database.path()).unwrap();
    let task = board
        .create_task("Login reliability", "Keep login recovery predictable")
        .unwrap();

    {
        let api = TaskAccessApi::new(&board);
        let decision = api
            .propose(
                task.id,
                RecordType::Decision,
                "Use a 20-second timeout",
                "coding-agent",
            )
            .unwrap();
        api.accept(decision.id).unwrap();
        let question = api
            .propose(
                task.id,
                RecordType::OpenQuestion,
                "Draft question",
                "coding-agent",
            )
            .unwrap();
        api.edit_and_accept(question.id, "Which callers own retry?")
            .unwrap();
        let change = api
            .propose(task.id, RecordType::Change, "Draft metrics", "coding-agent")
            .unwrap();
        api.edit_and_accept(change.id, "Add timeout instrumentation")
            .unwrap();
        let rejected = api
            .propose(
                task.id,
                RecordType::Change,
                "Discard this change",
                "coding-agent",
            )
            .unwrap();
        api.reject(rejected.id).unwrap();
        let sensitive = api
            .propose(
                task.id,
                RecordType::Decision,
                "Store API key in vault",
                "coding-agent",
            )
            .unwrap();
        api.accept(sensitive.id).unwrap();

        let before_approval = api.compose_handoff(task.id, "v1", "coding-agent").unwrap();
        assert_eq!(before_approval.goal, "Keep login recovery predictable");
        assert_eq!(before_approval.decisions, vec!["Use a 20-second timeout"]);
        assert_eq!(
            before_approval.open_questions,
            vec!["Which callers own retry?"]
        );
        assert_eq!(
            before_approval.next_steps,
            vec!["Add timeout instrumentation"]
        );

        api.approve_sensitive_release(sensitive.id).unwrap();
        let approved = api.compose_handoff(task.id, "v1", "coding-agent").unwrap();
        assert_eq!(approved.schema_version, "v1");
        assert_eq!(
            approved.decisions,
            vec!["Use a 20-second timeout", "Store API key in vault"]
        );
    }
    drop(board);

    let reopened = Board::open(database.path()).unwrap();
    let persisted = TaskAccessApi::new(&reopened)
        .compose_handoff(task.id, "v1", "coding-agent")
        .unwrap();
    assert_eq!(persisted.goal, "Keep login recovery predictable");
    assert_eq!(
        persisted.decisions,
        vec!["Use a 20-second timeout", "Store API key in vault"]
    );
    assert_eq!(persisted.open_questions, vec!["Which callers own retry?"]);
    assert_eq!(persisted.next_steps, vec!["Add timeout instrumentation"]);
}

#[test]
fn api_submission_keeps_stale_conflicts_pending_and_revocation_persists() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut board = Board::open(database.path()).unwrap();
    let task = board
        .create_task("Login reliability", "Keep login recovery predictable")
        .unwrap();

    let confirmed_id;
    {
        let api = TaskAccessApi::new(&board);
        let confirmed = api
            .propose(
                task.id,
                RecordType::Decision,
                "Use a 20-second timeout",
                "risky-agent",
            )
            .unwrap();
        confirmed_id = confirmed.id;
        api.accept(confirmed.id).unwrap();
        let risky = api
            .propose_from_version(
                task.id,
                RecordType::Decision,
                "Use a 60-second timeout",
                "risky-agent",
                1,
            )
            .unwrap();
        assert!(risky.is_stale);
        assert!(risky.is_conflict);
        assert_eq!(risky.conflicts_with, Some(confirmed_id));
        let pending = api.pending(task.id).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, risky.id);
        assert_eq!(
            board.resume(task.id).unwrap().confirmed[0].content,
            "Use a 20-second timeout"
        );

        api.revoke_session("risky-agent").unwrap();
        assert!(api
            .what_just_happened(task.id, "v1", "risky-agent")
            .is_err());
        assert!(LoopbackRelay
            .request_heartbeat(&api, task.id, "v1", "risky-agent")
            .is_err());
        assert!(api.subscribe("risky-agent").is_err());
        assert!(api.compose_handoff(task.id, "v1", "risky-agent").is_err());
        assert!(api
            .propose(task.id, RecordType::Change, "Denied", "risky-agent")
            .is_err());
        assert!(api
            .propose_from_version(task.id, RecordType::Change, "Denied", "risky-agent", 2)
            .is_err());
    }
    drop(board);

    let reopened = Board::open(database.path()).unwrap();
    let api = TaskAccessApi::new(&reopened);
    let pending = api.pending(task.id).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].content, "Use a 60-second timeout");
    assert!(pending[0].is_stale);
    assert!(pending[0].is_conflict);
    assert_eq!(pending[0].conflicts_with, Some(confirmed_id));
    assert!(api
        .what_just_happened(task.id, "v1", "risky-agent")
        .is_err());
    assert!(api.compose_handoff(task.id, "v1", "active-agent").is_ok());
    assert!(api
        .propose(task.id, RecordType::Change, "Active work", "active-agent")
        .is_ok());
}
