"""
Run this from any Claude Code session to load shared context from synclined.

Read:
  python claude_context.py

Propose (after analysis):
  python claude_context.py propose decision "Use connection pooling for DB queries"
  python claude_context.py propose change "Refactored auth middleware to be stateless"
"""

import sys
from synclined_client import SynclinedClient

sync = SynclinedClient(actor="claude-code")

if len(sys.argv) == 1:
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
            flag = " [STALE]" if p["is_stale"] else ""
            print(f"  [{p['id']}] {p['content']}{flag}")
        print()
        print(f"  Accept: python claude_context.py accept <id>")

elif sys.argv[1] == "propose" and len(sys.argv) == 4:
    _, _, kind, content = sys.argv
    pid = sync.propose(kind, content)
    print(f"proposed (id={pid}, pending review): {content}")
    print(f"  Accept: python claude_context.py accept {pid}")

elif sys.argv[1] == "accept" and len(sys.argv) == 3:
    pid = int(sys.argv[2])
    sync.accept(pid)
    print(f"accepted id={pid}")

else:
    print(__doc__)
