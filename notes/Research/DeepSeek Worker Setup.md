# DeepSeek worker setup

October 8, 2026. Tooling milestone; game acceptance remains at the existing Queen checkpoint.

## Configured and checked

- Installed and enabled `codex-on-crack@codex-on-crack`, version `2.2.2+codex.20261002`, source commit `9dee6ca9aaa73e0008123eff872871e88b94a12a`. All 71 installed source files matched the pinned checkout.
- Installed Codex Router `0.6.0`, source commit `444a72809d1609020516e14438c2075c5f4bda8c`, at `%USERPROFILE%\.codex\tools\codex-router`. Its Windows scheduled task and authenticated loopback health checks pass.
- the owner entered the DeepSeek credential through the router's hidden local prompt. Credential contents were not read into chat or added to this project.
- the owner chose DeepSeek V4.1 Flash, then requested max reasoning. The selected route is `deepseek/deepseek-v4.1-flash`; both the router's per-model effort and the generated `crack_builder` role use `max`.
- The lead remains `gpt-6-astra`; its reasoning setting is preserved. The role permits scoped implementation/tests, with evidence returned to the lead for review. No global crack policy was installed.
- Router doctor: 88 checks, zero failures. Warnings concern other providers/local models that are not configured. Crack doctor: `static-ready`, configured role in sync, zero problems.

## Verification boundary

This is configuration readiness, not demonstrated delegation. Flash is offered through explicit local operator selection; it has no completed native-delegation certification here. No paid installation probes or model requests were made. After fully quitting and reopening Codex, the first real authorized restoration task must establish the child's actual provider, model, reasoning effort, tool execution, and returned evidence before relying on it.

Catalog metadata lists image input, 1,048,576 context tokens, and low/high/max effort for Flash. These are advertised properties, not measured results or account entitlement proof. No throughput or 100-agent speedup is established.

## Windows installation findings

The packaged Codex app redirects nominal `AppData\Local` writes into its private package cache. A file-handle path probe confirmed the router source physically lived there, while the unpackaged Windows scheduled task looked at the nominal location and failed with `MODULE_NOT_FOUND`. Relocating the stable checkout outside AppData resolved source visibility without widening permissions.

The moved virtual environment retained the old path in `litellm.exe`. Reinstalling the same pinned LiteLLM wheel with its checked-in hashes regenerated that launcher. The next install and final doctor passed. Router source files remain unmodified.

## Local evidence and next step

Private, local installation records are under `%USERPROFILE%\.codex\tools\codex-on-crack-install-record`, including `final-verification.json`. Configuration snapshots and raw diagnostic reports remain outside this project. Do not publish their contents or router capability URLs.

Role setup/undo receipt: `%USERPROFILE%\.codex\crack-backups\20261008T231352Z-24b3ef40\receipt.json`.

Useful read-only checks from the router checkout:

```powershell
node src/doctor.mjs --json
node src/control.mjs subagents explain deepseek/deepseek-v4.1-flash --json
```

The first command's full report is private diagnostic material; use a sanitized summary when sharing results. Restart Codex before assigning the first real worker task. Continue the game from [[Current Progress]] afterward; this tooling work did not change the save, game code, or gameplay acceptance.

## First real work, October 8, 5:34 p.m. Pacific

Real delegated restoration work now produces source changes, notes and test receipts. Workers' own matching session files record `deepseek/deepseek-v4.1-flash` with `max` effort. The crafting builder returned a narrow Quit39/6 implementation with saved RED/GREEN evidence and a 179-pass ordinary suite. A separately scoped worker independently reproduced the old failures, checked the new logic and returned a review. Recovery-guide author/reviewer workers also returned verified artifacts.

This establishes observed model/effort selection and functioning delegated tool work. Session metadata's generic `model_provider=openai` label does not identify a particular upstream provider per request. No throughput benchmark or 100-agent scaling claim follows. Several workers ended turns with intermediate status before completing their assignment; the lead explicitly resumed them and reviewed their eventual artifacts. Finished children were interrupted/marked done. Game acceptance remains tracked separately in [[../Current Progress]].
