import httpx


class SynclinedError(RuntimeError):
    pass


class SynclinedClient:
    def __init__(self, base_url: str = "http://localhost:3000", actor: str = "agent"):
        self.base = base_url
        self.actor = actor

    def _get(self, path: str, **params) -> dict:
        try:
            r = httpx.get(f"{self.base}{path}", params=params)
        except httpx.ConnectError:
            raise SynclinedError(f"Cannot reach synclined at {self.base} — is the server running?\n  cargo run --bin synclined-server")
        r.raise_for_status()
        return r.json()

    def _post(self, path: str, body: dict | None = None) -> httpx.Response:
        try:
            r = httpx.post(f"{self.base}{path}", json=body)
        except httpx.ConnectError:
            raise SynclinedError(f"Cannot reach synclined at {self.base} — is the server running?\n  cargo run --bin synclined-server")
        r.raise_for_status()
        return r

    def context(self) -> dict:
        return self._get("/context", actor=self.actor)

    def propose(self, kind: str, content: str) -> int:
        r = self._post("/propose", {"kind": kind, "content": content, "actor": self.actor})
        return r.json()["id"]

    def accept(self, proposal_id: int) -> None:
        self._post(f"/accept/{proposal_id}")

    def propose_and_accept(self, kind: str, content: str) -> int:
        pid = self.propose(kind, content)
        self.accept(pid)
        return pid
