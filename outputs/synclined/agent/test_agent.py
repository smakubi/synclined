"""
Text-mode test for the synclined voice agent integration.

Replaces mic/speaker with stdin/stdout so the full loop can be verified
without audio hardware:
  - reads shared context from synclined
  - runs a GPT-4o conversation from scripted lines
  - extracts decisions/changes and writes them back
  - prints the updated context so you can confirm the write-back worked

Usage:
  # terminal 1
  cargo run --bin synclined-server

  # terminal 2
  python test_agent.py
"""

from openai import OpenAI
from synclined_client import SynclinedClient
import json

client = OpenAI()

SCRIPTED_TURNS = [
    "Hi, I want to confirm we're shipping the new feature on Friday.",
    "The API should use streaming responses to keep latency low.",
    "One open question: do we need a fallback if streaming fails?",
    "Great, I think that covers it. Goodbye.",
]


def build_system_prompt(ctx: dict) -> str:
    changes = "\n".join(f"- {c}" for c in ctx["recent_changes"]) or "None yet."
    return f"""\
You are a helpful voice assistant.

SHARED CONTEXT (approved decisions and changes):
Goal: {ctx['goal']}
History:
{changes}

Keep replies short — this is a voice conversation."""


def extract_records(messages: list[dict]) -> list[dict]:
    resp = client.chat.completions.create(
        model="gpt-4o",
        temperature=0,
        response_format={"type": "json_object"},
        messages=[
            {
                "role": "system",
                "content": (
                    "Extract key decisions, changes, and open questions from this "
                    "call transcript. Return JSON: "
                    "{\"records\": [{\"kind\": \"decision\"|\"change\"|\"open_question\", "
                    "\"content\": \"one concise sentence\"}]}. "
                    "Omit records with no real information."
                ),
            },
            *messages,
        ],
    )
    return json.loads(resp.choices[0].message.content).get("records", [])


def run_test():
    sync = SynclinedClient(actor="voice-agent")

    # ── 1. load shared context ─────────────────────────────────────────────
    ctx = sync.context()
    print("=" * 60)
    print("BEFORE CALL — context from synclined:")
    print(f"  task_id : {ctx['task_id']}")
    print(f"  goal    : {ctx['goal']}")
    print(f"  history : {ctx['recent_changes'] or '(empty)'}")
    print("=" * 60)

    system   = build_system_prompt(ctx)
    messages = []

    # ── 2. scripted conversation ───────────────────────────────────────────
    print("\n--- conversation (text mode) ---\n")
    for user_text in SCRIPTED_TURNS:
        print(f"user      : {user_text}")
        messages.append({"role": "user", "content": user_text})

        resp = client.chat.completions.create(
            model="gpt-4o",
            max_tokens=120,
            messages=[{"role": "system", "content": system}, *messages],
        )
        reply = resp.choices[0].message.content
        messages.append({"role": "assistant", "content": reply})
        print(f"assistant : {reply}\n")

    # ── 3. extract and write back ──────────────────────────────────────────
    print("--- extracting records from call ---\n")
    records = extract_records(messages)

    for rec in records:
        pid = sync.propose_and_accept(rec["kind"], rec["content"])
        print(f"[synclined] {rec['kind']:15s} (id={pid}): {rec['content']}")

    # ── 4. verify the write-back ───────────────────────────────────────────
    ctx_after = sync.context()
    print("\n" + "=" * 60)
    print("AFTER CALL — context from synclined:")
    print(f"  task_id : {ctx_after['task_id']}")
    print(f"  goal    : {ctx_after['goal']}")
    for c in ctx_after["recent_changes"]:
        print(f"  • {c}")
    print("=" * 60)


if __name__ == "__main__":
    run_test()
