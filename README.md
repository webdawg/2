# Defined Dataset Transport

A research prototype exploring whether large amounts of data can be reconstructed from a tiny
transmitted payload, by pointing into vast data repositories that both ends compute independently
instead of transmitting.

## The idea

Two machines share a known spec: the same generation processes (closed-form formulas, constant
expansions like π, seeded algorithms — and possibly, later, trained models) that let each side
produce identical output completely independently. None of that output ever crosses the wire — it's
regenerated locally on both ends from the shared spec, so it can be made arbitrarily large at zero
transmission cost.

Given a target file, the sending side searches its own copy of these repositories for a
`{dataset, coordinate, primer}` — a starting point, a formula/dataset id, and the extra parameters
needed to condition it — that reproduces the target exactly. Only that small tuple is transmitted.
The receiving side runs the same generation process against the same repositories to reproduce the
original data.

The motivating scenario is two supercomputers at opposite ends of an extremely low-bandwidth,
high-latency link (framed as interplanetary/interstellar) — see [`INTENT.md`](INTENT.md) for the full
thought experiment, including the one hard constraint this project has to respect: **a primer can only
ever address as much data as its own information content allows, regardless of how much compute or
time either side has.** That's not a limitation to engineer around — it's the boundary that defines
what's actually possible here, and characterizing it for real data is the point of the research.

## Status

Early research prototype, not a finished result. What exists today:

- `server/` — a TCP server exposing three closed-form deterministic generators (an LCG, a SHA-256 hash
  chain, and a raw polynomial) behind a small custom wire protocol.
- `client/` — a brute-force search tool that queries the server for random `{coordinate, params}`
  combinations looking for an exact match against a target byte sequence, to empirically probe how
  primer discovery behaves.

Brute-force search over these generators is a baseline for validating the protocol and search
mechanics — it is **not** expected to be the eventual discovery method, and it's already hitting the
information-theoretic wall by design (see [`COGNITION.md`](COGNITION.md) for what's been found running
it so far).

## Documentation map

This project keeps its *why*, *how*, and *research process* in separate, cross-linked docs — read them
in this order:

- [`INTENT.md`](INTENT.md) — why this project exists: the motivating thought experiment and the hard
  constraint the design has to respect.
- [`SPEC.md`](SPEC.md) — how it's built: terminology, architecture, and open technical questions.
- [`COGNITION.md`](COGNITION.md) — the research methodology: the empirical build-search-learn loop this
  project runs on, and a standing summary of what's been learned so far. Individual working-session
  notes live as `SESSION_<date>.md` files at the repo root.
- [`APPLICATIONS.md`](APPLICATIONS.md) — what kinds of real target data could plausibly fit the hard
  constraint above, and which are ruled out.

## Building and running

Requires a Rust toolchain (`rustc` + `cargo`) — see [`CLAUDE.md`](CLAUDE.md) for install notes. From
the repo root (a Cargo workspace):

```
cargo build
cargo test
cargo run --bin server -- [port]      # default port 7878, binds 127.0.0.1
cargo run --bin client -- --target "hi" --addr 127.0.0.1:7878
```

## License

[AGPL-3.0-or-later](LICENSE).
