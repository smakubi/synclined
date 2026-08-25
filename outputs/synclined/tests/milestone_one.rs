use synclined_core::{Board, ProposalOutcome, RecordType};

#[test]
fn persists_accepted_and_rejected_proposals_for_an_independent_reader() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut board = Board::open(database.path()).unwrap();
    let task = board
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();

    let accepted = board
        .propose(
            task.id,
            RecordType::OpenQuestion,
            "Is ten seconds intended?",
            "agent-a",
        )
        .unwrap();
    board.review(accepted.id, ProposalOutcome::Accept).unwrap();

    let rejected = board
        .propose(
            task.id,
            RecordType::Change,
            "Set timeout to thirty seconds",
            "agent-a",
        )
        .unwrap();
    board.review(rejected.id, ProposalOutcome::Reject).unwrap();
    drop(board);

    let resumed = Board::open(database.path()).unwrap();
    let snapshot = resumed.resume(task.id).unwrap();

    assert_eq!(snapshot.goal, "Keep session behavior unchanged");
    assert_eq!(snapshot.confirmed.len(), 1);
    assert_eq!(snapshot.confirmed[0].content, "Is ten seconds intended?");
    assert_eq!(snapshot.rejected.len(), 1);
    assert_eq!(
        snapshot.rejected[0].content,
        "Set timeout to thirty seconds"
    );
}
