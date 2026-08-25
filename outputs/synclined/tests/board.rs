use synclined_core::{Board, RecordType, TaskAccessApi};

#[test]
fn board_view_shows_goal_and_pending_proposals_then_reviews_through_engine() {
    let database = tempfile::NamedTempFile::new().unwrap();
    let mut engine = Board::open(database.path()).unwrap();
    let task = engine
        .create_task("Fix login timeout", "Keep session behavior unchanged")
        .unwrap();
    let proposal = engine
        .propose(task.id, RecordType::Change, "Raise timeout", "cloud-code")
        .unwrap();

    let api = TaskAccessApi::new(&engine);
    let view = synclined_core::BoardView::load(&api, task.id).unwrap();
    assert_eq!(view.goal, "Keep session behavior unchanged");
    assert_eq!(view.pending[0].content, "Raise timeout");

    view.accept(&api, proposal.id).unwrap();
    assert!(synclined_core::BoardView::load(&api, task.id)
        .unwrap()
        .pending
        .is_empty());
    assert_eq!(engine.resume(task.id).unwrap().confirmed.len(), 1);

    let rejected = engine
        .propose(
            task.id,
            RecordType::OpenQuestion,
            "Is it intended?",
            "cloud-code",
        )
        .unwrap();
    let api = TaskAccessApi::new(&engine);
    api.reject(rejected.id).unwrap();
    assert_eq!(engine.resume(task.id).unwrap().rejected.len(), 1);

    let edited = engine
        .propose(task.id, RecordType::Change, "Raise timeout", "cloud-code")
        .unwrap();
    let api = TaskAccessApi::new(&engine);
    api.edit_and_accept(edited.id, "Raise timeout after test evidence")
        .unwrap();
    assert_eq!(
        engine.resume(task.id).unwrap().confirmed[1].content,
        "Raise timeout after test evidence"
    );
}
