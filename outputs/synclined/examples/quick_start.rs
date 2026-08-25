use synclined_core::{Board, RecordType, TaskAccessApi};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut board = Board::open(":memory:")?;
    let task = board.create_task("First task", "Ship a reviewed task state")?;

    let api = TaskAccessApi::new(&board);
    let decision = api.propose(
        task.id,
        RecordType::Decision,
        "Use the versioned Task Access API",
        "quick-start",
    )?;
    api.accept(decision.id)?;

    let heartbeat = api.what_just_happened(task.id, "v1", "quick-start")?;
    println!("SyncLined quick start");
    println!("schema: {}", heartbeat.schema_version);
    println!("goal: {}", heartbeat.goal);
    for change in heartbeat.recent_changes {
        println!("approved: {change}");
    }

    Ok(())
}
