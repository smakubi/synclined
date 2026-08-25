use synclined_core::{Board, ProposalOutcome, RecordType};

#[test]
fn relay_returns_a_versioned_heartbeat_after_approval() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut board = Board::open(database.path()).unwrap();
    let task = board.create_task("Fix login timeout", "Keep session behavior unchanged").unwrap();
    let proposal = board.propose(task.id, RecordType::Change, "Raise timeout after evidence", "cloud-code").unwrap();
    board.review(proposal.id, ProposalOutcome::Accept).unwrap();

    let api = synclined_core::TaskAccessApi::new(&board);
    let relay = synclined_core::LoopbackRelay;
    let response = relay.request_heartbeat(&api, task.id, "v1", "phone-client").unwrap();
    assert_eq!(response.schema_version, "v1");
    assert_eq!(response.recent_changes, vec!["Raise timeout after evidence"]);
}
