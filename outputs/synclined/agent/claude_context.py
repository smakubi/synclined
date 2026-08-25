"""
Run this from any Claude Code session to load shared context from synclined.

Read:
  python claude_context.py

Handoff summary:
  python claude_context.py handoff

Propose (pending human review):
  python claude_context.py propose decision "Use connection pooling for DB queries"
  python claude_context.py propose change "Refactored auth middleware to be stateless"

Review pending proposals:
  python claude_context.py accept <id>
  python claude_context.py reject <id>
"""

import sys
from synclined_client import SynclinedClient

sync = SynclinedClient(actor="claude-code")

cmd = sys.argv[1] if len(sys.argv) > 1 else "read"

if cmd == "read":
    ctx = sync.context()
    print(f"task_id : {ctx['task_id']}")
    print(f"goal    : {ctx['goal']}")
    print()
    if ctx["recent_changes"]:
        print("Approved context (visible to all agents):")
        for c in ctx["recent_changes"]:
            print(f"  • {c}")
    else:
        print("No approved context yet.")

    pending = sync._get("/pending")["pending"]
    if pending:
        print()
        print("Pending (awaiting review):")
        for p in pending:
            flags = []
            if p["is_stale"]:    flags.append("STALE")
            if p["is_conflict"]: flags.append("CONFLICT")
            if p["sensitive"] != "not_required": flags.append(f"SENSITIVE:{p['sensitive']}")
            flag_str = f"  [{', '.join(flags)}]" if flags else ""
            print(f"  [{p['id']}] ({p['actor']} / {p['kind']}){flag_str}")
            print(f"       {p['content']}")
        print()
        print("  python claude_context.py accept <id>")
        print("  python claude_context.py reject <id>")

elif cmd == "handoff":
    h = sync._get("/handoff", actor="claude-code")
    print(f"=== Handoff (task {h['task_id']}) ===")
    print(f"Goal: {h['goal']}")
    if h["decisions"]:
        print("\nDecisions:")
        for d in h["decisions"]: print(f"  • {d}")
    if h["open_questions"]:
        print("\nOpen questions:")
        for q in h["open_questions"]: print(f"  ? {q}")
    if h["next_steps"]:
        print("\nNext steps:")
        for s in h["next_steps"]: print(f"  → {s}")

elif cmd == "propose" and len(sys.argv) == 4:
    _, _, kind, content = sys.argv
    pid = sync.propose(kind, content)
    print(f"proposed (id={pid}, pending review): {content}")
    print(f"  python claude_context.py accept {pid}")
    print(f"  python claude_context.py reject {pid}")

elif cmd == "accept" and len(sys.argv) == 3:
    pid = int(sys.argv[2])
    sync.accept(pid)
    print(f"accepted id={pid}")

elif cmd == "reject" and len(sys.argv) == 3:
    pid = int(sys.argv[2])
    sync._post(f"/reject/{pid}")
    print(f"rejected id={pid}")

else:
    print(__doc__)
