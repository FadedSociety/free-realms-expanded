"""Optional asynchronous prefix of the existing private datagram capture."""
import json
import queue
import threading
from pathlib import Path


class LiveCapture:
    """Never wait for disk on submit; the final harness capture stays authoritative."""

    def __init__(self, run, queue_limit=4096):
        if queue_limit <= 0:
            raise ValueError("Live queue must be bounded and positive")
        self.run = Path(run)
        self.index = self.run / "frames/live-index.jsonl"
        self.status = self.run / "frames/live-status.json"
        if self.index.exists() or self.status.exists():
            raise ValueError("Live capture requires a new run")
        self.pending = queue.Queue(maxsize=queue_limit)
        self.stop = threading.Event()
        self.error = None
        self.written = 0
        self.thread = threading.Thread(target=self._write, name="live-capture", daemon=True)
        self.thread.start()

    def submit(self, record, data):
        if self.stop.is_set() or self.error is not None:
            return
        try:
            self.pending.put_nowait((record, data))
        except queue.Full:
            self.error = "Live queue full; live prefix is incomplete; final capture remains authoritative"

    def _status(self, state):
        payload = {"schema_version": 1, "state": state, "authoritative": False,
                   "written_frames": self.written, "pending_frames": self.pending.qsize(),
                   "incomplete": self.error is not None or not self.pending.empty(),
                   "error": self.error}
        temporary = self.status.with_suffix(".tmp")
        temporary.write_text(json.dumps(payload) + "\n", encoding="utf-8")
        temporary.replace(self.status)

    def _write(self):
        try:
            with self.index.open("x", encoding="utf-8", newline="\n") as index:
                self._status("recording")
                while not self.stop.is_set():
                    try:
                        record, data = self.pending.get(timeout=0.1)
                    except queue.Empty:
                        if self.error is not None:
                            self._status("incomplete")
                        continue
                    # Publish a record only after its exact final-format file is closed.
                    (self.run / record["path"]).write_bytes(data)
                    index.write(json.dumps(record, separators=(",", ":")) + "\n")
                    index.flush()
                    self.written += 1
                    if self.error is not None:
                        self._status("incomplete")
        except Exception as error:
            self.error = "Live writer failed: " + str(error)
        finally:
            try:
                self._status("closed" if self.error is None else "incomplete")
            except OSError:
                pass

    def close(self):
        # Stop before final authoritative frame writes; do not drain a disk backlog.
        self.stop.set()
        self.thread.join()
        return self.error


def read_live_frames(run):
    """Snapshot complete published lines; a concurrent partial last line is ignored."""
    data = (Path(run) / "frames/live-index.jsonl").read_bytes()
    return [json.loads(line) for line in data[:data.rfind(b"\n") + 1].splitlines()]
