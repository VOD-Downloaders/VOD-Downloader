"""
ratecheck.py, interactive rate-limit probe.

Fires requests at a target req/s that you adjust live from the keyboard, like a
slider. A single shared token bucket paces a pool of workers, so the AGGREGATE
rate is what's bounded (same model as a parallel segment downloader). Surfaces
Retry-After and flags Cloudflare 1015 specifically.

    pip install requests
    ./ratecheck.py https://host/path/seg00001.ts \
        -H "Referer: https://host/" \
        --rate 5 --workers 8

Live keys:
    + / =   rate +1 req/s        ] / [   rate +/-10 req/s
    } / {   burst +/-1 token     p       pause / resume
    b       toggle auto-backoff  r       reset counters
    q       quit

Run it in a real terminal. If curses can't init (e.g. piped output) it falls
back to a line-based mode with the same keys (arrow keys unsupported there).

NOTE: This was written by AI (claude) for testing
"""
from __future__ import annotations

import argparse
import datetime
import email.utils
import sys
import threading
import time
from collections import Counter, deque
from dataclasses import dataclass, field

try:
    import requests
except ImportError:
    sys.exit("This needs 'requests':  pip install requests")

# --------------------------------------------------------------------------- #
# Token bucket: aggregate pacing shared by every worker.
# --------------------------------------------------------------------------- #
class TokenBucket:
    def __init__(self, rate: float, capacity: float) -> None:
        self._rate = float(rate)
        self._capacity = float(capacity)
        self._tokens = float(capacity)
        self._last = time.monotonic()
        self._lock = threading.Lock()

    def _refill_locked(self) -> None:
        now = time.monotonic()
        self._tokens = min(self._capacity, self._tokens + (now - self._last) * self._rate)
        self._last = now

    def set_rate(self, rate: float) -> None:
        with self._lock:
            self._refill_locked()
            self._rate = max(0.0, float(rate))

    def get_rate(self) -> float:
        with self._lock:
            return self._rate

    def set_capacity(self, cap: float) -> None:
        with self._lock:
            self._capacity = max(1.0, float(cap))
            self._tokens = min(self._tokens, self._capacity)

    def get_capacity(self) -> float:
        with self._lock:
            return self._capacity

    def acquire(self, stop: threading.Event) -> bool:
        """Block until one token is free. Returns False if stopped first."""
        while not stop.is_set():
            with self._lock:
                self._refill_locked()
                if self._tokens >= 1.0:
                    self._tokens -= 1.0
                    return True
                rate = self._rate
                deficit = 1.0 - self._tokens
            wait = (deficit / rate) if rate > 0 else 0.1
            time.sleep(min(max(wait, 0.001), 0.1))
        return False

# --------------------------------------------------------------------------- #
# Shared state.
# --------------------------------------------------------------------------- #
@dataclass
class Stats:
    sent: int = 0
    codes: Counter = field(default_factory=Counter)
    rate_limited: int = 0
    errors: int = 0
    last_status: str = "-"
    last_retry_after: str = "-"
    _completions: deque = field(default_factory=lambda: deque(maxlen=4096))
    _lock: threading.Lock = field(default_factory=threading.Lock)

    def record(self, code: str, limited: bool, error: bool, retry_after: str | None) -> None:
        with self._lock:
            self.sent += 1
            self.codes[code] += 1
            self.last_status = code
            if limited:
                self.rate_limited += 1
            if error:
                self.errors += 1
            if retry_after is not None:
                self.last_retry_after = retry_after
            self._completions.append(time.monotonic())

    def achieved_rate(self, window: float = 5.0) -> float:
        cutoff = time.monotonic() - window
        with self._lock:
            while self._completions and self._completions[0] < cutoff:
                self._completions.popleft()
            n = len(self._completions)
        return n / window

    def reset(self) -> None:
        with self._lock:
            self.sent = 0
            self.codes.clear()
            self.rate_limited = 0
            self.errors = 0
            self.last_status = "-"
            self.last_retry_after = "-"
            self._completions.clear()

    def snapshot(self) -> dict:
        with self._lock:
            return {
                "sent": self.sent,
                "codes": dict(self.codes),
                "rate_limited": self.rate_limited,
                "errors": self.errors,
                "last_status": self.last_status,
                "last_retry_after": self.last_retry_after,
            }

@dataclass
class Control:
    burst: int
    backoff: bool = False
    _backoff_until: float = 0.0
    _lock: threading.Lock = field(default_factory=threading.Lock)

    def arm_backoff(self, seconds: float) -> None:
        with self._lock:
            self._backoff_until = max(self._backoff_until, time.monotonic() + seconds)

    def backoff_remaining(self) -> float:
        with self._lock:
            return max(0.0, self._backoff_until - time.monotonic())

class Log:
    """Thread-safe ring of recent event lines for the UI."""
    def __init__(self, n: int = 500) -> None:
        self._dq: deque[str] = deque(maxlen=n)
        self._lock = threading.Lock()

    def add(self, msg: str) -> None:
        line = f"{time.strftime('%H:%M:%S')} {msg}"
        with self._lock:
            self._dq.append(line)

    def tail(self, n: int) -> list[str]:
        with self._lock:
            return list(self._dq)[-n:]

# --------------------------------------------------------------------------- #
# Helpers.
# --------------------------------------------------------------------------- #
def parse_retry_after(value: str | None) -> float | None:
    """Retry-After is either delta-seconds or an HTTP-date. Returns seconds."""
    if not value:
        return None
    value = value.strip()
    if value.isdigit():
        return float(value)
    try:
        dt = email.utils.parsedate_to_datetime(value)
        if dt is not None:
            now = datetime.datetime.now(dt.tzinfo) if dt.tzinfo else datetime.datetime.now()
            return max(0.0, (dt - now).total_seconds())
    except (TypeError, ValueError):
        pass
    return None

def is_limited(status: int, body_snippet: str) -> tuple[bool, bool]:
    """Returns (rate_limited, is_1015)."""
    cf1015 = "1015" in body_snippet and "error code" in body_snippet.lower()
    limited = status in (429, 503) or status == 1015 or cf1015
    return limited, (status == 1015 or cf1015)

# --------------------------------------------------------------------------- #
# Worker.
# --------------------------------------------------------------------------- #
def worker(session, args, headers, bucket, control, stats, log,
           stop: threading.Event, paused: threading.Event) -> None:
    while not stop.is_set():
        if paused.is_set() or control.backoff_remaining() > 0:
            time.sleep(0.05)
            continue
        if not bucket.acquire(stop):
            break
        try:
            resp = session.request(
                args.method, args.url, headers=headers,
                timeout=args.timeout, allow_redirects=False, stream=True,
            )
            status = resp.status_code
            snippet = ""
            if status != 200:
                try:
                    snippet = next(resp.iter_content(2048, decode_unicode=True)) or ""
                    if isinstance(snippet, bytes):
                        snippet = snippet.decode("utf-8", "replace")
                except (StopIteration, Exception):
                    snippet = ""
            resp.close()

            limited, cf1015 = is_limited(status, snippet)
            ra_raw = resp.headers.get("Retry-After")
            ra_sec = parse_retry_after(ra_raw)
            ra_disp = f"{ra_sec:.0f}s" if ra_sec is not None else (ra_raw or "-")

            stats.record(str(status), limited, False, ra_disp if limited else None)

            if limited:
                tag = "1015" if cf1015 else "limit"
                log.add(f"RATE-LIMITED [{tag}] status={status} "
                        f"Retry-After={ra_disp} (sent={stats.sent})")
                if control.backoff:
                    delay = ra_sec if ra_sec is not None else 5.0
                    control.arm_backoff(delay)
                    log.add(f"backing off {delay:.0f}s (all workers)")
            elif status >= 400:
                log.add(f"status={status}")
        except requests.RequestException as e:
            stats.record("ERR", False, True, None)
            log.add(f"error: {type(e).__name__}: {e}")
            time.sleep(0.2)

# --------------------------------------------------------------------------- #
# Key handling shared by both UIs.
# --------------------------------------------------------------------------- #
def apply_key(ch: str, bucket, control, stats, log, paused) -> bool:
    """Mutate state from a keypress. Returns True to quit."""
    if ch in ("q", "Q"):
        return True
    elif ch in ("+", "="):
        bucket.set_rate(bucket.get_rate() + 1)
    elif ch in ("-", "_"):
        bucket.set_rate(max(0, bucket.get_rate() - 1))
    elif ch == "]":
        bucket.set_rate(bucket.get_rate() + 10)
    elif ch == "[":
        bucket.set_rate(max(0, bucket.get_rate() - 10))
    elif ch == "}":
        control.burst += 1
        bucket.set_capacity(control.burst)
    elif ch == "{":
        control.burst = max(1, control.burst - 1)
        bucket.set_capacity(control.burst)
    elif ch in ("p", "P"):
        (paused.clear if paused.is_set() else paused.set)()
        log.add("resumed" if not paused.is_set() else "paused")
    elif ch in ("b", "B"):
        control.backoff = not control.backoff
        log.add(f"auto-backoff {'ON' if control.backoff else 'OFF'}")
    elif ch in ("r", "R"):
        stats.reset()
        log.add("counters reset")
    return False

def status_text(args, bucket, control, stats, paused) -> str:
    s = stats.snapshot()
    state = "PAUSED" if paused.is_set() else (
        f"BACKOFF {control.backoff_remaining():.0f}s" if control.backoff_remaining() > 0 else "running")
    codes = " ".join(f"{k}:{v}" for k, v in sorted(s["codes"].items())) or "-"
    return (
        f"target={bucket.get_rate():.0f}/s  achieved={stats.achieved_rate():.1f}/s  "
        f"burst={int(bucket.get_capacity())}  workers={args.workers}  "
        f"backoff={'on' if control.backoff else 'off'}  [{state}]\n"
        f"sent={s['sent']}  limited={s['rate_limited']}  errors={s['errors']}  "
        f"last={s['last_status']}  Retry-After={s['last_retry_after']}\n"
        f"codes: {codes}"
    )

# --------------------------------------------------------------------------- #
# curses UI.
# --------------------------------------------------------------------------- #
def run_curses(stdscr, args, bucket, control, stats, log, stop, paused, threads):
    import curses
    curses.curs_set(0)
    stdscr.nodelay(True)
    stdscr.keypad(True)
    keymap = {curses.KEY_UP: "+", curses.KEY_DOWN: "-",
              curses.KEY_RIGHT: "]", curses.KEY_LEFT: "["}

    while not stop.is_set():
        try:
            ch = stdscr.getch()
            if ch != -1:
                key = keymap.get(ch, chr(ch) if 0 <= ch < 256 else "")
                if key and apply_key(key, bucket, control, stats, log, paused):
                    break

            stdscr.erase()
            h, w = stdscr.getmaxyx()

            def put(y, x, text, attr=0):
                if 0 <= y < h:
                    stdscr.addnstr(y, x, text, max(0, w - x - 1), attr)

            put(0, 0, f" ratecheck  {args.method} {args.url}", curses.A_REVERSE)
            for i, line in enumerate(status_text(args, bucket, control, stats, paused).split("\n")):
                put(2 + i, 0, line, curses.A_BOLD if i == 0 else 0)
            put(6, 0, "+/- rate1  ]/[ rate10  }/{ burst  p pause  b backoff  r reset  q quit",
                curses.A_DIM)
            put(8, 0, "── events " + "─" * max(0, w - 11))
            tail = log.tail(max(0, h - 10))
            for i, line in enumerate(tail):
                put(9 + i, 0, line)
            stdscr.refresh()
        except curses.error:
            pass
        time.sleep(0.1)

    stop.set()

# --------------------------------------------------------------------------- #
# Fallback line-based UI (no curses / not a TTY).
# --------------------------------------------------------------------------- #
def run_fallback(args, bucket, control, stats, log, stop, paused, threads):
    def key_reader():
        try:
            import termios, tty, select
            fd = sys.stdin.fileno()
            old = termios.tcgetattr(fd)
            try:
                tty.setcbreak(fd)
                while not stop.is_set():
                    if select.select([sys.stdin], [], [], 0.2)[0]:
                        ch = sys.stdin.read(1)
                        if apply_key(ch, bucket, control, stats, log, paused):
                            stop.set()
                            return
            finally:
                termios.tcsetattr(fd, termios.TCSADRAIN, old)
        except Exception:
            # Windows or no termios: best-effort line input.
            try:
                import msvcrt  # type: ignore
                while not stop.is_set():
                    if msvcrt.kbhit():
                        ch = msvcrt.getwch()
                        if apply_key(ch, bucket, control, stats, log, paused):
                            stop.set()
                            return
                    time.sleep(0.1)
            except Exception:
                while not stop.is_set():  # input unavailable; just run
                    time.sleep(0.5)

    kt = threading.Thread(target=key_reader, daemon=True)
    kt.start()
    print("Keys: +/- rate  ]/[ x10  }/{ burst  p pause  b backoff  r reset  q quit\n")
    seen = 0
    try:
        while not stop.is_set():
            new = log.tail(50)
            for line in new[seen if seen <= len(new) else 0:]:
                print(line)
            seen = len(log.tail(50))
            print("  " + status_text(args, bucket, control, stats, paused).replace("\n", " | "),
                  end="\r", flush=True)
            time.sleep(0.5)
    except KeyboardInterrupt:
        pass
    finally:
        stop.set()
        print()

# --------------------------------------------------------------------------- #
# Main.
# --------------------------------------------------------------------------- #
def parse_headers(raw: list[str]) -> dict:
    headers = {}
    for item in raw or []:
        if ":" not in item:
            sys.exit(f"Bad header (need 'Key: Value'): {item!r}")
        k, v = item.split(":", 1)
        headers[k.strip()] = v.strip()
    return headers

def main() -> None:
    p = argparse.ArgumentParser(description="Interactive rate-limit probe.")
    p.add_argument("url")
    p.add_argument("-H", "--header", action="append", default=[],
                   help="Header 'Key: Value' (repeatable, curl-style)")
    p.add_argument("-X", "--method", default="GET", help="HTTP method (default GET; HEAD avoids bodies)")
    p.add_argument("--rate", type=float, default=2.0, help="Initial target req/s")
    p.add_argument("--burst", type=int, default=1, help="Token-bucket capacity (burst allowance)")
    p.add_argument("--workers", type=int, default=8, help="Concurrent worker threads")
    p.add_argument("--timeout", type=float, default=10.0, help="Per-request timeout (s)")
    p.add_argument("--simple", action="store_true", help="Force line-based UI (no curses)")
    args = p.parse_args()

    headers = parse_headers(args.header)
    bucket = TokenBucket(args.rate, max(1, args.burst))
    control = Control(burst=max(1, args.burst))
    stats = Stats()
    log = Log()
    stop = threading.Event()
    paused = threading.Event()

    session = requests.Session()
    adapter = requests.adapters.HTTPAdapter(
        pool_connections=args.workers, pool_maxsize=args.workers)
    session.mount("http://", adapter)
    session.mount("https://", adapter)

    threads = [
        threading.Thread(target=worker,
                         args=(session, args, headers, bucket, control, stats, log, stop, paused),
                         daemon=True)
        for _ in range(args.workers)
    ]
    for t in threads:
        t.start()
    log.add(f"started: {args.method} {args.url}  rate={args.rate}/s workers={args.workers}")

    use_curses = (not args.simple) and sys.stdout.isatty()
    try:
        if use_curses:
            try:
                import curses
                curses.wrapper(run_curses, args, bucket, control, stats, log, stop, paused, threads)
            except Exception:
                run_fallback(args, bucket, control, stats, log, stop, paused, threads)
        else:
            run_fallback(args, bucket, control, stats, log, stop, paused, threads)
    except KeyboardInterrupt:
        pass
    finally:
        stop.set()
        for t in threads:
            t.join(timeout=1.0)
        snap = stats.snapshot()
        print("\n── summary ──")
        print(f"sent={snap['sent']}  rate-limited={snap['rate_limited']}  errors={snap['errors']}")
        print("codes: " + (" ".join(f"{k}:{v}" for k, v in sorted(snap['codes'].items())) or "-"))
        print(f"last Retry-After: {snap['last_retry_after']}")

if __name__ == "__main__":
    main()
