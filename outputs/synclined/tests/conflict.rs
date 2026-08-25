use synclined_core::{Board, ProposalOutcome, RecordType};

#[test]
fn conflicting_proposal_is_flagged_without_changing_confirmed_state() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();

    let confirmed = engine
        .propose(
            task.id,
            RecordType::Decision,
            "Keep the timeout at ten seconds",
            "developer",
        )
        .unwrap();
    engine
        .review(confirmed.id, ProposalOutcome::Accept)
        .unwrap();

    let proposal = engine
        .propose(
            task.id,
            RecordType::Decision,
            "Raise the timeout to thirty seconds",
            "cloud-code",
        )
        .unwrap();

    assert!(proposal.is_conflict);
    assert_eq!(proposal.conflicts_with, Some(confirmed.id));
    assert_eq!(
        engine.resume(task.id).unwrap().confirmed[0].content,
        "Keep the timeout at ten seconds"
    );

    drop(engine);
    let reopened = Board::open(database.path()).unwrap();
    let pending = reopened.pending(task.id).unwrap();
    assert!(pending[0].is_conflict);
    assert_eq!(pending[0].conflicts_with, Some(confirmed.id));
}

#[test]
fn open_questions_do_not_create_possible_conflicts() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();

    let confirmed = engine
        .propose(
            task.id,
            RecordType::OpenQuestion,
            "Is ten seconds intended?",
            "developer",
        )
        .unwrap();
    engine
        .review(confirmed.id, ProposalOutcome::Accept)
        .unwrap();

    let proposal = engine
        .propose(
            task.id,
            RecordType::OpenQuestion,
            "Should we measure failures?",
            "cloud-code",
        )
        .unwrap();

    assert!(!proposal.is_conflict);
    assert_eq!(proposal.conflicts_with, None);
}
