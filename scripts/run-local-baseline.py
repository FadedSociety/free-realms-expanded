"""Bounded client/emulator baseline in disposable copies, with local UDP tracing.

Uses Universal Modder for configuration backup and GPU window capture. Sends no
mouse/keyboard input. The supplied launcher arguments are read locally, not stored
in this repository. Run with lab/tools/universal-modder-env/Scripts/python.exe.
"""
import argparse
import hashlib
import json
import os
import re
import select
import shlex
import shutil
import socket
import subprocess
import sys
import time
import threading
import zlib
from collections import Counter
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from um import win
from asset_delivery import AssetDelivery
from journal_text_overlay import (validate_journal_text, apply_journal_text,
                                  validate_character_select_title, apply_character_select_title)
from lua_probe_overlay import validate_lua_probe, apply_lua_probe
from live_capture import LiveCapture
sys.path.insert(1, str(Path(__file__).with_name("bundle")))
from client_overlay import install as install_client_overlay

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--seconds", type=int, default=90)
parser.add_argument("--rust-reference", type=Path, help="Use the Rust gateway with a prepared private reference directory")
parser.add_argument("--state-file", type=Path, help="Rust character save file, outside the source repository")
parser.add_argument("--content", type=Path, help="Private gameplay content for the Rust gateway")
parser.add_argument("--native-login", action="store_true", help="Use loopback Rust login and the original character creator/selection UI")
parser.add_argument("--identity-file", type=Path, help="Separate native-created character identity file")
parser.add_argument("--journal-text", type=Path, help="Opt-in versioned private en_US journal text package matching --content")
parser.add_argument("--character-select-title", type=Path, help="Opt-in versioned one-operand character selection title movie repair")
parser.add_argument("--lua-probe", action="store_true", help="Replace per-run UI/ScriptsBase.bin with syntax-checked Lua source and an automatic quest-offer probe")
parser.add_argument("--live-capture", action="store_true", help="Publish a non-authoritative live UDP prefix in the new run via a background writer")
parser.add_argument("--keep-client", action="store_true", help="Keep the full per-run client copy (default: delete it after the run, keeping client/Logs)")
parser.add_argument("--gamepad", action="store_true", help="Drive this run's client with an XInput controller (scripts/launcher/gamepad_bridge.py, layout in scripts/launcher/gamepad.json)")
parser.add_argument("--native-hook", metavar="SET", help="Attach the Frida native hook set (see native_hook.py --list) to this run's client")
parser.add_argument("--live-log", action=argparse.BooleanOptionalAction, default=True, help="Write server/login packet traces and open the merged live log in a console beside the client (default on; --no-live-log to disable)")
parser.add_argument("--dev-tools", action="store_true", help="Opt in to local developer entity teleport commands for this test run")
parser.add_argument("--drop-server-sequence", type=int, help="Drop the first transmission of this reliable sequence to test recovery")
run_options = parser.parse_args()
if run_options.native_login and (not run_options.rust_reference or not run_options.content):
    parser.error("--native-login requires --rust-reference and --content")
if run_options.dev_tools and (not run_options.rust_reference or not run_options.content):
    parser.error("--dev-tools requires --rust-reference and --content")
if not 10 <= run_options.seconds <= 3600:
    parser.error("--seconds must be between 10 and 3600")
if run_options.drop_server_sequence is not None and not 0 <= run_options.drop_server_sequence <= 65535:
    parser.error("--drop-server-sequence must fit a 16-bit sequence")

lab = Path(__file__).resolve().parents[2] / "lab"
if run_options.journal_text and not run_options.native_login:
    parser.error("--journal-text requires --native-login (explicit Locale8)")
if run_options.character_select_title and not run_options.native_login:
    parser.error("--character-select-title requires --native-login")
journal_text = validate_journal_text(run_options.journal_text, run_options.content,
                                    lab / "working/OSFR.Client/OSFR Client", lab)
character_select_title = validate_character_select_title(run_options.character_select_title, lab)
lua_probe = validate_lua_probe(lab / "content/lua-probe" if run_options.lua_probe else None,
                              lab / "working/OSFR.Client/OSFR Client", lab)
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
run = lab / "runs" / ("client-baseline-" + stamp)
client = run / "client"
server = run / "server"
shutil.copytree(lab / "working/OSFR.Client/OSFR Client", client)
apply_journal_text(journal_text, client, run, lab)
# World Builder text: authored strings become rows in this run's client locale copy.
if run_options.dev_tools and (lab / "world/text.json").is_file():
    subprocess.run([sys.executable, "-I", str(Path(__file__).with_name("build_world_locale.py")),
                    "--world", str(lab / "world"), "--locale", str(client / "locale")], check=True)
lua_probe_overlay = apply_lua_probe(lua_probe, client, run, lab)
if run_options.native_login:
    # The launcher's client overlay: pinned community creator tables that remove
    # unsupported skin7/8 and extra faces, where the client opens them, plus the custom/
    # dressing room. Future choices only; saved identities remain untouched.
    install_client_overlay(client)
if run_options.rust_reference:
    server.mkdir()
else:
    shutil.copytree(lab / "working/OSFR.Server/OSFR Server", server)
apply_character_select_title(character_select_title, client, run, lab)
# Official Microsoft redistributable component, added only to the run copy.
runtime = lab / "tools/directx-june2010/x86/d3dx9_31.dll"
if runtime.is_file():
    shutil.copy2(runtime, client / runtime.name)
(run / "frames").mkdir()
before = run / "config-before"
before.mkdir()
for path in client.glob("*.ini"):
    shutil.copy2(path, before / path.name)
subprocess.run([str(lab / "tools/universal-modder-env/Scripts/um.exe"),
                "backup", "create", str(before), "--name", "freerealms-" + stamp], check=True)

front = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
front.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 8 * 1024 * 1024)
front.bind(("127.0.0.1", 0))
back = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
back.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 8 * 1024 * 1024)
back.bind(("127.0.0.1", 0))
with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as reserve:
    reserve.bind(("127.0.0.1", 0))
    server_port = reserve.getsockname()[1]
state_file = (run_options.state_file or lab / "state/character.json").resolve()
if run_options.rust_reference:
    reference = run_options.rust_reference.resolve()
    gateway_binary = lab.parent / "project/prototype/osfr-tools/target/release/osfr-gateway.exe"
    server_command = [str(gateway_binary), f"127.0.0.1:{server_port}", "--reference", str(reference),
                      "--state", str(state_file), "--seconds", str(run_options.seconds + 10),
                      # The client's own tables: style cards (42/10, 42/12) refuse without them.
                      "--resources", str(client / "Resources")]
    if run_options.content:
        server_command += ["--content", str(run_options.content.resolve())]
    if run_options.dev_tools:
        server_command += ["--dev-tools", "--client-logs", str(client / "Logs")]
    # World Builder live layer: in-game builds (placed NPCs, marks, authored content) persist here.
    if run_options.dev_tools:
        server_command += ["--world", str(lab / "world"),
                           "--client-locale", str(lab / "working/OSFR.Client/OSFR Client/locale")]
    # Ground heights (osfr-terrain output) for enemy movement, teleports and the Studio map.
    if (lab / "terrain").is_dir() and any((lab / "terrain").glob("zone_*.hgt")):
        server_command += ["--terrain", str(lab / "terrain")]
    if run_options.live_log:
        server_command += ["--trace", str(run / "server-trace.jsonl")]
    if run_options.native_login:
        identity_file = (run_options.identity_file or lab / "state/opening-identity.json").resolve()
        identity_file.parent.mkdir(parents=True, exist_ok=True)
        server_command += ["--identity", str(identity_file)]
        # W55 login integration: login and gateway share one account store and a
        # per-run local handoff key that signs the selected-character binding.
        account_store_file = identity_file.with_name("accounts.json")
        handoff_key_file = run / "handoff.key"
        handoff_key_file.write_bytes(os.urandom(16))
        server_command += ["--account-store", str(account_store_file),
                           "--handoff-key-file", str(handoff_key_file)]
    if state_file.exists():
        shutil.copy2(state_file, run / "character-before.json")
else:
    config = json.loads((server / "Config.json").read_text(encoding="utf-8-sig"))
    config[0]["Port"] = server_port
    (server / "Config.json").write_text(json.dumps(config, indent=2), encoding="utf-8")
    server_command = [str(server / "OSFRServer.exe"), "--verbose"]
endpoint = f"127.0.0.1:{front.getsockname()[1]}"
# OSFR v1.0 release documents serving manifest.crc locally. Its published
# CRC/length pair matches zlib.crc32(Assets_manifest.txt); use this run's own
# preserved manifest so the client does not request an unrelated asset set.
manifest_bytes = (client / "Assets_manifest.txt").read_bytes()
manifest_crc = f"{zlib.crc32(manifest_bytes)},{len(manifest_bytes)}\r\n".encode("ascii")
http_requests = []
http_counts = Counter()
asset_delivery = AssetDelivery(lab, manifest_bytes)
asset_proofs = {}
http_metrics_lock = threading.Lock()


class AssetsHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        path = self.path.split("?", 1)[0]
        body = {"/manifest.crc": manifest_crc,
                "/Assets_manifest.txt": manifest_bytes,
                "/manifest.txt.z": asset_delivery.manifest_wire}.get(path)
        if body is None:
            try:
                delivered = asset_delivery.get(self.path)
                if delivered is not None:
                    body, proof = delivered
                    with http_metrics_lock:
                        asset_proofs[proof["name"]] = proof
            except (OSError, ValueError) as error:
                with http_metrics_lock:
                    http_counts[("<rejected_asset_source>", 0)] += 1
        status = 200 if body is not None else 404
        with http_metrics_lock:
            http_counts[(path, status)] += 1
            if len(http_requests) < 1000:
                http_requests.append({"path": path, "status": status})
        self.send_response(status)
        self.send_header("Content-Type", "application/octet-stream")
        self.send_header("Content-Length", str(len(body) if body is not None else 0))
        self.end_headers()
        if body is not None:
            self.wfile.write(body)

    def log_message(self, *_args):
        pass


class AssetServer(ThreadingHTTPServer):
    daemon_threads = True

    def handle_error(self, _request, _address):
        with http_metrics_lock:
            http_counts[("<connection_error>", 0)] += 1


asset_http = AssetServer(("127.0.0.1", 0), AssetsHandler)
threading.Thread(target=asset_http.serve_forever, daemon=True).start()
asset_url = f"http://127.0.0.1:{asset_http.server_port}/"
ini = (client / "ClientConfig.ini").read_text(encoding="utf-8-sig")
ini = re.sub(r"(?m)^Server=.*$", "Server=" + endpoint, ini)
ini = re.sub(r"https?://[^\s\r\n]+", "http://127.0.0.1:9/", ini)
ini = re.sub(r"(?m)^IndirectServerAddress=.*$", "IndirectServerAddress=" + asset_url, ini)
ini = re.sub(r"(?m)^Address=lvsadp[^\r\n]+", "Address=127.0.0.1", ini)
(client / "Baseline.ini").write_text(ini, encoding="utf-8")
options = (client / "UserOptions.ini").read_text(encoding="utf-8-sig")
for key, value in {"WindowedWidth": "1024", "WindowedHeight": "768", "Mode": "Windowed",
                   "Maximized": "0", "Master": "0.000000"}.items():
    options = re.sub(rf"(?m)^{key}=.*$", key + "=" + value, options)
(client / "UserOptions.ini").write_text(options, encoding="utf-8")
launch = (client / "FreeRealms.bat").read_text()
arguments = shlex.split(re.search(r"FreeRealms\.exe\s+([^\r\n]+)", launch).group(1), posix=False)
arguments = ["inifile=Baseline.ini" if a.startswith("inifile=") else
             "Server=" + endpoint if a.startswith("Server=") else a for a in arguments]
if run_options.native_login:
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as login_reserve:
        login_reserve.bind(("127.0.0.1", 0))
        login_port = login_reserve.getsockname()[1]
    login_front = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    login_front.bind(("127.0.0.1", 0))
    login_back = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    login_back.bind(("127.0.0.1", 0))
    login_endpoint = f"127.0.0.1:{login_front.getsockname()[1]}"
    login_command = [str(lab.parent / "project/prototype/osfr-tools/target/release/osfr-login.exe"),
                     "--bind", f"127.0.0.1:{login_port}", "--gateway", endpoint,
                     "--resources", str(client / "Resources"), "--identity", str(identity_file),
                     "--key-file", str(lab / "references/rust-login/client-login-key.bin"),
                     "--state", str(state_file), "--account-store", str(account_store_file),
                     "--handoff-key-file", str(handoff_key_file),
                     "--seconds", str(run_options.seconds + 10)]
    if run_options.live_log:
        login_command += ["--trace", str(run / "login-trace.jsonl")]
    arguments = [a for a in arguments if a.split("=", 1)[0].lower() not in ("guid", "ticket", "key", "cassessionid", "sessionid", "server")]
    arguments += ["Server=" + login_endpoint, "SessionId=local-session", "Internationalization:Locale=8"]
arguments.append("GameAccountManagement:UriRoot=http://127.0.0.1:9/")
startup = subprocess.STARTUPINFO()
startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
startup.wShowWindow = 0
processes, handles, frames, frame_bytes, screenshots = [], [], [], [], []
captured_bytes = 0
result = {"run_directory": str(run), "started_utc": datetime.now(timezone.utc).isoformat(),
          "server_kind": "rust_gateway" if run_options.rust_reference else "supplied_emulator",
          "server_executable_sha256": hashlib.sha256(Path(server_command[0]).read_bytes()).hexdigest(),
          "client_endpoint": endpoint, "server_port": server_port,
          "asset_url": asset_url, "asset_manifest_crc": manifest_crc.decode("ascii").strip(),
          "asset_manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
          "relay_receive_buffer_bytes": back.getsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF),
          "input_automation": False, "configuration_backup": str(before)}
result["developer_tools_enabled"] = run_options.dev_tools
if run_options.live_log:
    result["live_log_enabled"] = True
    result["live_log"] = str(run / "live.log")
if run_options.rust_reference:
    result.update(state_file=str(state_file), reference_directory=str(reference))
    if run_options.content:
        result["content_directory"] = str(run_options.content.resolve())
if run_options.native_login:
    result.update(native_login=True, identity_file=str(identity_file), login_endpoint=login_endpoint, login_port=login_port)
    if identity_file.exists():
        shutil.copy2(identity_file, run / "identity-before.json")
if journal_text:
    result["journal_text_overlay"] = journal_text["proof"]
if character_select_title:
    result["character_select_title_overlay"] = character_select_title["proof"]
if lua_probe_overlay:
    result["lua_probe_overlay"] = lua_probe_overlay
    result["lua_probe_log"] = lua_probe_overlay["run_log"]
live_capture = LiveCapture(run) if run_options.live_capture else None
started = time.monotonic()
try:
    launches = [("server", server_command, server)]
    if run_options.native_login:
        launches.append(("login", login_command, server))
    launches.append(("client", [str(client / "FreeRealms.exe"), *arguments], client))
    for name, command, cwd in launches:
        out = (run / (name + "-stdout.log")).open("wb")
        err = (run / (name + "-stderr.log")).open("wb")
        handles += [out, err]
        process = subprocess.Popen(command, cwd=cwd, stdout=out, stderr=err,
                                   startupinfo=startup, creationflags=subprocess.CREATE_NO_WINDOW)
        processes.append((name, process))
        result[name + "_pid"] = process.pid
        if name == "client" and run_options.gamepad:
            # First in the list, so it is stopped last: it exits by itself
            # when the client does, releasing any held keys first.
            processes.insert(0, ("gamepad", subprocess.Popen(
                [sys.executable, "-I", str(Path(__file__).with_name("launcher") / "gamepad_bridge.py"),
                 "--pid", str(process.pid)],
                stdout=(run / "gamepad-stdout.log").open("wb"), stderr=subprocess.STDOUT,
                creationflags=subprocess.CREATE_NO_WINDOW)))
        if name == "client" and run_options.native_hook:
            # Frida native hooks on this run's client copy; hits go to native-hook.jsonl.
            subprocess.Popen(
                [sys.executable, str(Path(__file__).with_name("native_hook.py")), str(run),
                 "--set", run_options.native_hook],
                stdout=(run / "native-hook-stdout.log").open("wb"),
                stderr=subprocess.STDOUT,
                creationflags=subprocess.CREATE_NO_WINDOW,
            )
        if name == "client" and run_options.live_log:
            # Start the viewer directly in its own console. `cmd /c start ...` was
            # misread by the shell and raised "Pick an app" dialogs instead.
            # Show it without activating it, so it never takes focus from the game
            # (it did in run 204017 and the input driver refused to type).
            viewer_startup = subprocess.STARTUPINFO()
            viewer_startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            viewer_startup.wShowWindow = 4  # SW_SHOWNOACTIVATE
            subprocess.Popen(
                [sys.executable, str(Path(__file__).with_name("live_log.py")), str(run),
                 "--client-pid", str(process.pid), "--parent-pid", str(os.getpid())],
                cwd=Path(__file__).resolve().parent.parent,
                creationflags=subprocess.CREATE_NEW_CONSOLE,
                startupinfo=viewer_startup,
            )
        if run_options.rust_reference and name in ("server", "login"):
            deadline = time.monotonic() + 10
            while True:
                if process.poll() is not None:
                    raise RuntimeError(f"{name} failed before client launch; see {run / (name + '-stderr.log')}")
                if '"event":"listening"' in (run / (name + "-stdout.log")).read_text(encoding="utf-8", errors="replace"):
                    break
                if time.monotonic() >= deadline:
                    raise RuntimeError(f"{name} did not become ready before client launch")
                time.sleep(0.05)
    peer = None
    login_peer = None
    negotiated_crc = 0
    dropped = False
    capture_times = iter([12, 30, 50, 80])
    next_capture = next(capture_times, 999)
    while time.monotonic() - started < run_options.seconds:
        if run_options.rust_reference:
            for service_name, service_process in processes:
                if service_name != "client" and service_process.poll() is not None:
                    raise RuntimeError(f"{service_name} exited during preview; see its logs in {run}")
        elapsed = time.monotonic() - started
        client_process = next(p for name, p in processes if name == "client")
        if client_process.poll() is not None:
            result["client_exit_code"] = client_process.returncode
            break
        sockets = [front, back] + ([login_front, login_back] if run_options.native_login else [])
        for sock in select.select(sockets, [], [], 0.05)[0]:
            try:
                data, address = sock.recvfrom(65535)
            except ConnectionResetError as error:
                # Windows reports a UDP peer's closed port on the next recv.
                # A login handoff can close its ephemeral socket normally;
                # this is not evidence that the separate game client exited.
                if getattr(error, "winerror", None) != 10054:
                    raise
                continue
            forwarded = True
            service = "gateway"
            if sock is front:
                peer = address
                back.sendto(data, ("127.0.0.1", server_port))
                direction = "client-to-server"
            elif sock is back:
                if data[:2] == b"\x00\x02" and len(data) == 21:
                    negotiated_crc = data[10]
                elif (run_options.drop_server_sequence is not None and not dropped
                      and negotiated_crc and data[:2] in (b"\x00\x09", b"\x00\x0d")):
                    body = data[3:-negotiated_crc]
                    if data[2]:
                        inflater = zlib.decompressobj()
                        body = inflater.decompress(body, 65536)
                        if not inflater.eof or inflater.unused_data:
                            raise RuntimeError("invalid outgoing compressed frame in loss check")
                    if len(body) >= 2 and int.from_bytes(body[:2], "big") == run_options.drop_server_sequence:
                        forwarded = False
                        dropped = True
                        result["intentional_dropped_sequence"] = run_options.drop_server_sequence
                if peer and forwarded:
                    front.sendto(data, peer)
                direction = "server-to-client"
            elif sock is login_front:
                login_peer = address
                login_back.sendto(data, ("127.0.0.1", login_port))
                direction, service = "client-to-server", "login"
            else:
                if login_peer:
                    login_front.sendto(data, login_peer)
                direction, service = "server-to-client", "login"
            path = f"frames/{len(frames):06d}.bin"
            # The legacy server bursts a whole fragmented message without a
            # send window. Disk I/O here can overflow the relay's receive queue.
            # Keep this bounded trace in memory and persist after forwarding.
            frame_bytes.append(data)
            captured_bytes += len(data)
            frames.append({"time": elapsed, "direction": direction, "service": service, "path": path,
                           "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
                           "forwarded": forwarded,
                           "opcode": int.from_bytes(data[:2], "big")})
            if len(frames) > 100000 or captured_bytes > 256 * 1024 * 1024:
                raise RuntimeError("baseline trace reached its count/256-MiB byte bound")
            if live_capture is not None:
                live_capture.submit(frames[-1], data)
        if elapsed >= next_capture:
            try:
                windows = [p for p in win.processes("FreeRealms") if p["Id"] == result["client_pid"]]
                result["client_windows"] = windows
                if windows:
                    path = run / f"client-{int(next_capture):02d}.png"
                    win.shot(str(path), hwnd=windows[0]["Hwnd"], scale=0.5, timeout=10)
                    screenshots.append(str(path))
            except (Exception, SystemExit) as error:
                result.setdefault("capture_errors", []).append(str(error))
            next_capture = next(capture_times, float("inf"))
finally:
    result["elapsed_seconds"] = time.monotonic() - started
    for name, process in reversed(processes):
        if process.poll() is None:
            process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
        result[name + "_stopped"] = process.poll() is not None
    front.close()
    back.close()
    if run_options.native_login:
        login_front.close()
        login_back.close()
        if identity_file.exists():
            shutil.copy2(identity_file, run / "identity-after.json")
    asset_http.shutdown()
    asset_http.server_close()
    for handle in handles:
        handle.close()
    if live_capture is not None:
        live_error = live_capture.close()
        if live_error:
            print(live_error)
    for record, data in zip(frames, frame_bytes):
        (run / record["path"]).write_bytes(data)
    if run_options.rust_reference and state_file.exists():
        shutil.copy2(state_file, run / "character-after.json")
    result["frames"] = frames
    result["screenshots"] = screenshots
    with http_metrics_lock:
        result["asset_http_requests"] = list(http_requests)
        result["asset_http_counts"] = [{"path": path, "status": status, "count": count}
                                      for (path, status), count in http_counts.items()]
        result["asset_delivery_proofs"] = list(asset_proofs.values())
    (run / "manifest.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    (lab / "manifests/latest-client-baseline.json").write_text(
        json.dumps({k: v for k, v in result.items() if k != "frames"}, indent=2), encoding="utf-8")
    print(json.dumps({k: v for k, v in result.items() if k not in ("frames", "asset_http_requests", "asset_http_counts", "asset_delivery_proofs")}, indent=2))
    print("Captured datagrams:", len(frames))
    # The per-run client copy is ~1.8 GB and duplicates lab/working; 134 runs
    # filled the disk on 2026-10-14. Keep only client/Logs (evidence) unless
    # --keep-client is given.
    if not run_options.keep_client and client.is_dir():
        for entry in client.iterdir():
            if entry.name == "Logs":
                continue
            try:
                shutil.rmtree(entry) if entry.is_dir() else entry.unlink()
            except OSError as error:
                print(f"client copy cleanup skipped {entry.name}: {error}")
