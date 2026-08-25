"""
Voice agent wired to synclined for shared context.

STT : OpenAI Whisper
LLM : OpenAI GPT-4o
TTS : OpenAI TTS (PCM, played via sounddevice)

Push-to-talk: press Enter to start recording, Enter again to stop.
Type 'q' + Enter at the prompt to end the call.
"""

import io
import json
import wave

import numpy as np
import sounddevice as sd
from openai import OpenAI

from synclined_client import SynclinedClient

SAMPLE_RATE = 16_000   # mic capture rate (Whisper accepts 16 kHz)
TTS_RATE    = 24_000   # OpenAI PCM TTS output rate
CHANNELS    = 1

client = OpenAI()      # reads OPENAI_API_KEY from env


# ── audio helpers ──────────────────────────────────────────────────────────────

def record_turn() -> bytes | None:
    """Push-to-talk. Returns raw WAV bytes, or None to end the call."""
    print("\n[press Enter to speak, or type 'q' then Enter to end call] ", end="", flush=True)
    if input().strip().lower() == "q":
        return None

    print("[recording … press Enter to stop]", flush=True)
    chunks = []

    with sd.InputStream(samplerate=SAMPLE_RATE, channels=CHANNELS,
                        dtype="int16",
                        callback=lambda data, *_: chunks.append(data.copy())):
        input()

    if not chunks:
        return b""   # caller skips empty turns

    audio = np.concatenate(chunks, axis=0)

    buf = io.BytesIO()
    with wave.open(buf, "wb") as wf:
        wf.setnchannels(CHANNELS)
        wf.setsampwidth(2)
        wf.setframerate(SAMPLE_RATE)
        wf.writeframes(audio.tobytes())
    return buf.getvalue()


def transcribe(audio_bytes: bytes) -> str:
    buf = io.BytesIO(audio_bytes)
    buf.name = "audio.wav"
    return client.audio.transcriptions.create(model="whisper-1", file=buf).text


def speak(text: str) -> None:
    response = client.audio.speech.create(
        model="tts-1",
        voice="alloy",
        input=text,
        response_format="pcm",   # raw signed 16-bit PCM at TTS_RATE
    )
    pcm = np.frombuffer(response.content, dtype=np.int16)
    sd.play(pcm, samplerate=TTS_RATE)
    sd.wait()


# ── synclined helpers ──────────────────────────────────────────────────────────

def build_system_prompt(ctx: dict) -> str:
    changes = "\n".join(f"- {c}" for c in ctx["recent_changes"]) or "None yet."
    return f"""\
You are a helpful voice assistant.

SHARED CONTEXT (approved decisions and changes):
Goal: {ctx['goal']}
History:
{changes}

Keep replies short — this is a voice conversation, not a chat.
When the caller says goodbye or ends the call, say a brief farewell."""


def extract_records(messages: list[dict]) -> list[dict]:
    """Ask the LLM to pull decisions / changes out of the conversation."""
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


# ── main call loop ─────────────────────────────────────────────────────────────

def run_call():
    sync    = SynclinedClient(actor="voice-agent")
    ctx     = sync.context()
    system  = build_system_prompt(ctx)
    messages: list[dict] = []

    print(f"\n[synclined] task_id={ctx['task_id']}  goal='{ctx['goal']}'")
    print("[synclined] approved context loaded into system prompt\n")

    # opening line
    opening = "Hello! I'm ready. How can I help you today?"
    print(f"assistant: {opening}")
    speak(opening)

    while True:
        audio = record_turn()
        if audio is None:
            break

        user_text = transcribe(audio)
        if not user_text.strip():
            continue
        print(f"user: {user_text}")

        messages.append({"role": "user", "content": user_text})

        resp = client.chat.completions.create(
            model="gpt-4o",
            max_tokens=150,
            messages=[{"role": "system", "content": system}, *messages],
        )
        reply = resp.choices[0].message.content
        messages.append({"role": "assistant", "content": reply})

        print(f"assistant: {reply}")
        speak(reply)

    # ── write back to synclined ────────────────────────────────────────────
    if not messages:
        print("\n[synclined] no conversation to write back")
        return

    print("\n[synclined] extracting records from call …")
    records = extract_records(messages)

    for rec in records:
        pid = sync.propose_and_accept(rec["kind"], rec["content"])
        print(f"[synclined] {rec['kind']} accepted (id={pid}): {rec['content']}")

    if not records:
        print("[synclined] nothing to write back")


if __name__ == "__main__":
    run_call()
