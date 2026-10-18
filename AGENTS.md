# AGENTS.md — APECSdev/apecs-guardian

Operating rules for editing this repository. This is a **node monitor that runs
on operators' machines** — the trust bar is the highest of any APECS repo.
Every rule cites a file.

## 1. Identity

| Item | Value | Source |
| --- | --- | --- |
| Binary | `apecs` (Windows: `apecs.exe`) | `Cargo.toml` `[[bin]]` |
| Lang | Rust 2021, tokio async | `Cargo.toml`, `src/main.rs` |
| Repo | `github.com:APECSdev/apecs-guardian` (PRIVATE) | `gh repo view` |
| Backend | `https://apecs.dev/v1/guardian/` | `src/comm/api.rs` `DEFAULT_BASE` |
| Auth | `x-apecs-key` header on every POST | `src/comm/api.rs` |
| Chains | Dash, Bitcoin, Bitcoin Cash, Avalanche, Dogecoin, Litecoin | `src/chains.rs` `Chain::ALL` |

## 2. Non-negotiables

1. **Never guess.** Cite a file path or command output.
2. **`cargo test` passes before any commit.** Currently 25 tests. New behavior
   ships with a test in the same commit.
3. **READ-ONLY invariant.** No code path may mutate the host or execute a
   free-text command. The `Action` enum (`src/cmd/action.rs`) is a CLOSED set:
   adding a variant is a reviewed change, never data-driven. **Never add
   `Install`, `Build`, or `Shell`.**
4. **Three-layer gate stays.** Any remote action must pass: (1) operator opt-in
   `--allow-actions`, (2) closed-enum deserialization, (3) Ed25519 signature +
   expiry (`src/auth/verify.rs`). Do not bypass or weaken a layer.
5. **Fail closed.** With the placeholder `CONTROL_PUBKEY_HEX` (all zeros),
   every signed action rejects. Baking in the real public key is a deliberate,
   one-time step (`apecs keygen`).
6. **Secrets never committed.** The control-plane PRIVATE key lives only in the
   APECS secret store; `*.pem`/`*.key`/`.env` are git-ignored.
7. **Outbound only.** No inbound listener / open port. Poll, don't serve.

## 3. Known limitations (do not "fix" silently)

- **`CONTROL_PUBKEY_HEX` is the all-zero placeholder.** Signed actions cannot
  authorize until the real public key from `apecs keygen` is baked in and the
  binary rebuilt. This is intentional (fail closed).
- **`NodeLogs` is a stub.** `execute()` reports the clamped intent but does not
  yet read a chain log file. Implementing it must stay read-only and bounded
  (`MAX_LOG_TAIL`).
- **Detection is name/PATH based.** A non-standard install dir or renamed
  binary won't be detected. Process-name matching (`dashd`, `bitcoind`, …) is
  per-chain in `src/chains/detect.rs`.
- **Avalanche uses its HTTP admin API** (no `avalanche-cli`); current detector
  only checks the `avalanchego` process. Querying node health is a later
  milestone.

## 4. Verification checklist

1. `cargo build --release` clean (no new warnings).
2. `cargo test` all pass; regression test for any fixed defect.
3. `git diff --stat` — only intended files moved.
4. If you touched `cmd/action.rs`, `auth/`, or `comm/monitor.rs`, state so
   explicitly — those are the trust boundary — and re-run the full suite.
5. Cross-compile check: `cargo build --release --target x86_64-pc-windows-gnu`
   must still produce `apecs.exe`.

## 5. Relationship to other repos

- `APECSdev/api` — serves `/v1/guardian/*` (register, telemetry, action poll).
- `apecs.dev` — renders the live node dashboard at `/guardian/<sessionid>`.
- `APECSdev/admin` — the control-plane UI that (with the private key) signs
  action requests.

Do not duplicate this source into those repos; they consume the API + the
signed-action envelope only.

## 6. Product tiers (brand direction, ratified 2026-10-17; rebranded 2026-10-18)

The brand is two AI-generated characters with avatars and bios (public
assets and copy live in the apecs.dev repo: `public/img/characters/`,
`pages/guardian/index.vue`). Tiers are licensed PER DEVICE:

- **Guardian — "The Watcher"** — this repo. READ-ONLY: reporting and
  suggestive. It watches, reports, and may RECOMMEND an action — a human
  (or an explicit signed control-plane request) executes. Free tier:
  monitoring ONLY, every device, no seat key. Creed: "See everything.
  Touch nothing."
- **Guru — "The Ascended"** — the AUTONOMOUS tier (paid, PRO): agentic
  management. It acts on the operator's behalf within the signed-action
  plane. Licensed per device (one signed seat each). NOT YET BUILT.
  Creed: "AI proposes, the signature disposes."

The former name **"Guardian Angel" is retired** — do not use it in new
copy; the Pro tier is Guru.

License NFTs (one per device, minted on the user's primary network) are
managed by the Human Resources surface at `apecs.dev/hr`; the control plane
verifies NFT ownership before signing Guru-tier actions.

NFT chain selection is ON HOLD (operator decision, 2026-10-18): the
operator is building a new network, RISCy (`https://riscy.network` — does
not resolve yet, verified 2026-10-18), which may be incorporated into
APECS when it launches later this year. Do not commit to a mint-chain
matrix until that decision lands.

Guru's intelligence lives in the CONTROL PLANE, never in this daemon. The
rule is: **AI proposes, the signature disposes.** An agent may watch
telemetry and suggest or auto-issue actions, but this daemon's trust model
does not change for it — closed `Action` enum, Ed25519 signature required,
fail closed. New enum variants (e.g. `RestartDaemon`) may be added over
time; a shell variant may NEVER be added. Do not add "AI" code to this
repo; Guru is a control-plane feature.
