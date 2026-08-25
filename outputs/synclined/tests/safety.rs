use synclined_core::{Board, RecordType};

#[test]
fn proposal_based_on_an_older_confirmed_state_is_flagged_for_review() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine.create_task("Fix login timeout", "Keep session behavior unchanged").unwrap();
    let proposal = engine.propose_from_version(task.id, RecordType::Change, "Raise timeout", "cloud-code", 0).unwrap();
    assert!(proposal.is_stale);
}
