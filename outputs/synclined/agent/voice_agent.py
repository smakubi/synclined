"""
Voice agent wired to synclined for shared context.

STT : OpenAI Whisper
LLM : OpenAI GPT-4o
TTS : OpenAI TTS (PCM, played via sounddevice)

Real-time VAD: mic listens continuously, speech detected by amplitude.
Say "goodbye" / "bye" / "that's all" to end the call, or press Ctrl-C.
"""

import io
import json
import wave

import numpy as np
import sounddevice as sd
from openai import OpenAI

from synclined_client import SynclinedClient

SAMPLE_RATE   = 16_000
TTS_RATE      = 24_000
CHANNELS      = 1
FRAME_MS      = 30
FRAME_SAMPLES = SAMPLE_RATE * FRAME_MS // 1000   # 480 samples per 30 ms frame
ENERGY_THRESH = 300    # RMS amplitude — raise if mic picks up too much background
SILENCE_LIMIT = 33     # ~1 s of silence after speech ends the turn
MIN_SPEECH    = 8      # discard clips shorter than ~240 ms (noise bursts)

FAREWELL = {"bye", "goodbye", "that's all", "end call", "hang up", "see you", "done"}

client = OpenAI()


# ── audio helpers ──────────────────────────────────────────────────────────────

def record_turn() -> bytes | None:
    """
    Listen continuously. Return WAV bytes when speech ends.
    Return b"" if the clip is too short (noise). Return None on Ctrl-C.
    """
    speech: list[bytes] = []
    in_speech = False
    silence_count = 0

    print("\n[listening …]", end="", flush=True)

    try:
        with sd.RawInputStream(
            samplerate=SAMPLE_RATE, channels=CHANNELS,
            dtype="int16", blocksize=FRAME_SAMPLES,
        ) as stream:
            while True:
                raw, _ = stream.read(FRAME_SAMPLES)
                frame = np.frombuffer(raw, dtype=np.int16)
                energy = np.sqrt(np.mean(frame.astype(np.float32) ** 2))

                if energy > ENERGY_THRESH:
                    if not in_speech:
                        print(" [speaking …]", end="", flush=True)
                    in_speech = True
                    silence_count = 0
                    speech.append(bytes(raw))
                elif in_speech:
                    speech.append(bytes(raw))
                    silence_count += 1
                    if silence_count >= SILENCE_LIMIT:
                        break   # enough silence — turn is over

    except KeyboardInterrupt:
        print()
        return None

    if len(speech) < MIN_SPEECH:
        return b""

    buf = io.BytesIO()
    with wave.open(buf, "wb") as wf:
        wf.setnchannels(CHANNELS)
        wf.setsampwidth(2)
        wf.setframerate(SAMPLE_RATE)
        wf.writeframes(b"".join(speech))
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
        response_format="pcm",
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


def is_farewell(text: str) -> bool:
    lower = text.lower()
    return any(word in lower for word in FAREWELL)


# ── main call loop ─────────────────────────────────────────────────────────────

def run_call():
    sync   = SynclinedClient(actor="voice-agent")
    ctx    = sync.context()
    system = build_system_prompt(ctx)
    messages: list[dict] = []

    print(f"\n[synclined] task_id={ctx['task_id']}  goal='{ctx['goal']}'")
    print("[synclined] approved context loaded into system prompt")
    print("[tip] speak naturally — say 'goodbye' or press Ctrl-C to end the call\n")

    opening = "Hello! I'm ready. How can I help you today?"
    print(f"assistant: {opening}")
    speak(opening)

    while True:
        audio = record_turn()

        if audio is None:          # Ctrl-C
            break
        if not audio:              # noise / too short
            continue

        user_text = transcribe(audio)
        print(f"\nuser: {user_text}")

        if not user_text.strip():
            continue

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

        if is_farewell(user_text):
            break

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
