import httpx


class SynclinedClient:
    def __init__(self, base_url: str = "http://localhost:3000", actor: str = "agent"):
        self.base = base_url
        self.actor = actor

    def context(self) -> dict:
        return httpx.get(f"{self.base}/context", params={"actor": self.actor}).json()

    def propose(self, kind: str, content: str) -> int:
        r = httpx.post(f"{self.base}/propose", json={
            "kind": kind,
            "content": content,
            "actor": self.actor,
        }).json()
        return r["id"]

    def accept(self, proposal_id: int) -> None:
        httpx.post(f"{self.base}/accept/{proposal_id}")

    def propose_and_accept(self, kind: str, content: str) -> int:
        pid = self.propose(kind, content)
        self.accept(pid)
        return pid
