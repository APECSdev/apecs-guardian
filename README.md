# APECS Guardian

*Systems engineering, handled.*

**APECS Guardian handles the systems engineering — processes, logs, telemetry,
dashboards — so builders don't have to.**

`apecs.exe` — the **APECS Guardian**: a read-only, hardened node monitor for
top-L1 operators. A single static binary that registers your node with the
APECS control plane and reports live telemetry to your dashboard at
`apecs.dev/guardian/<sessionid>`.

Monitored chains (auto-detected, read-only):
**Dash · Bitcoin · Bitcoin Cash · Avalanche · Dogecoin · Litecoin**

## Security model (read this first)

Guardian is **read-only by default**. Three independent layers prevent remote
code execution:

1. **Monitor-only default.** Out of the box the daemon only *collects* and
   *reports* telemetry. No code path executes a server instruction. A
   compromised API can do nothing but receive metrics.
2. **Closed action allowlist.** If (and only if) you pass `--allow-actions`,
   the control plane may request an operation — but it can only name a variant
   of a compile-time enum (`NodeStatus`, `NodeLogs`, `CollectMetrics`). There
   is **no `Install`, `Build`, or `Shell`** variant; a server payload can never
   become an arbitrary command.
3. **Signed actions.** Every action request must carry an Ed25519 signature
   over `(action, expiry, nonce)` from the APECS control-plane key. The daemon
   ships with APECS's **public** key baked in and fails closed until a real key
   is configured. A stolen API session cannot forge an action.

> There is no inbound listener — the daemon polls **outbound** only.

## Usage

```sh
apecs run --key <your-apecs-key>            # register + start monitoring
apecs snapshot                              # print one telemetry snapshot (no network)
apecs keygen                                # control-plane ONLY: generate the signing keypair
```

Flags: `--base <url>` (default `https://apecs.dev/v1/guardian/`),
`--interval <secs>` (default 60), `--allow-actions` (default off),
`--key` (or set `APECS_KEY`).

## Build

```sh
cargo build --release                                   # native
cargo build --release --target x86_64-pc-windows-gnu    # apecs.exe (Windows)
cargo test                                              # test suite
```

Release profile: LTO, `panic = "abort"`, stripped → ~3 MB single binary.
TLS via rustls (no OpenSSL), so it cross-compiles cleanly.

## Backend

Reports to `/v1/guardian/*` on the `APECSdev/api` Worker, authenticated with
`x-apecs-key` (the same key discipline as the ADB agent). The web dashboard is
served from `apecs.dev`.

## License

MIT — see [LICENSE](LICENSE). Copyright APECS Dev.
