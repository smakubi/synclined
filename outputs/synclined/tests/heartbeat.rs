use synclined_core::{Board, ProposalOutcome, RecordType};

#[test]
fn authorized_client_receives_only_confirmed_heartbeat_state() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut board = Board::open(database.path()).unwrap();
    let task = board.create_task("Fix login timeout", "Keep session behavior unchanged").unwrap();
    let proposal = board.propose(task.id, RecordType::Change, "Raise timeout after evidence", "cloud-code").unwrap();
    board.review(proposal.id, ProposalOutcome::Accept).unwrap();

    let summary = board.what_just_happened(task.id, "v1", "phone-client").unwrap();
    assert_eq!(summary.schema_version, "v1");
    assert_eq!(summary.goal, "Keep session behavior unchanged");
    assert_eq!(summary.recent_changes, vec!["Raise timeout after evidence"]);
}
