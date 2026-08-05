# COGNITION.md

Status: this document tracks *how* the project does research — using working
software as the way to empirically investigate primer discovery, rather than
trying to settle it on paper — and is the standing summary of what's been
learned running it. See `INTENT.md` for why the project exists and `SPEC.md`
for the terminology/architecture referenced below.

## Why software-first research

Client file encoding, and the primer discovery method itself, can't be defined
on paper — they depend on empirical results that don't exist yet. The approach
is a loop, not a one-time bring-up:

1. **Raise a reproducible stack.** Build one instance of the full stack (a
   defined dataset + its generation process + a Rust server) that can be stood
   up identically by anyone, anywhere — this is what makes later results
   trustworthy/repeatable rather than a one-off on a single machine.
2. **Search against it.** Drive that stack with random/systematic
   {coordinate, primer} trials and observe the output: how sensitive output is
   to small primer changes, whether nearby coordinates/primers produce related
   output, how large a primer has to get before it reliably reproduces a target
   byte sequence of a given length, etc.
3. **Derive the encoding/method from what's found.** Once it's known
   empirically what a {dataset, coordinate, primer} tuple needs to contain to
   be useful, and what search strategy actually finds one efficiently, the
   client file format and discovery method fall out of that — not the other
   way around.

As smarter search/discovery methods are tried, go back through steps 2-3 again.
This document's "Findings so far" and "Open research directions" sections
should get updated each time a pass through the loop changes the picture.

## Current substrate: closed-form generators as a stepping stone

The three generators implemented in `server/src/algorithms/` (LCG, SHA-256 hash
chain, quadratic polynomial) are **not** the "physical constants computed to
extreme precision" repositories described in `INTENT.md`'s motivating scenario.
They're a deliberately small, fast, easy-to-reason-about substrate for
developing and validating the search/discovery *mechanics* — the wire protocol,
the client file shape, and candidate search strategies — before committing to a
much larger and more expensive repository. Once the discovery pipeline is
proven out here, the substrate can be swapped for the real target repositories
(a π digit expansion, physical constants, etc.) without changing the
surrounding architecture.

## Findings so far (empirical)

- `TCP_NODELAY` was required to get realistic round-trip timing (see
  `server/src/lib.rs`, commit `050e343`) — otherwise Nagle's algorithm +
  delayed ACKs added tens of ms per request, badly distorting search-loop
  timing measurements.
- The brute-force random-coordinate/random-params search (`client/`) confirms
  `INTENT.md`'s "Known constraint" empirically at small scale: a 1-byte target
  matches in a few hundred attempts (~1/256 odds, as expected against these
  generators); a 2-byte target took 6,299 attempts against `lcg`; nothing
  longer than 2 bytes has been confirmed found yet. `hash-chain`'s search
  coordinate had to be capped (`MAX_HASH_CHAIN_COORDINATE`) because it has no
  jump-ahead and a random `u64` coordinate would never return in practice.
- This is the exponential wall from `INTENT.md`'s "Known constraint" showing up
  in miniature: uniform-random brute-force search over a generator with no
  exploitable structure is a bad discovery method past a couple of bytes,
  regardless of how much time it's given. It's useful as a baseline and for
  validating the protocol/client-file mechanics, not as the eventual discovery
  method.
- Full session narrative: see `SESSION_2026-08-03.md`.

## Open research directions

(Methodology-level — see `SPEC.md`'s Open questions for the architectural
unknowns these feed into.)

- Replace blind random search with something that exploits each generator's
  actual structure instead of guessing:
  - `polynomial` (`a*x^2 + b*x + c mod 256`) likely admits direct parameter
    recovery from a handful of sample points (interpolation) rather than
    random guessing.
  - `lcg` parameter recovery from observed output is a known technique in the
    cryptanalysis literature — worth applying here instead of brute force.
  - `hash-chain` (SHA-256-based) is deliberately one-way by design and
    probably isn't invertible this way. Worth deciding whether it's a useful
    substrate for the *sender-side search* at all, versus keeping it only for
    receiver-side/protocol mechanics testing.
- Once a smarter search method exists for the closed-form generators,
  re-measure the empirical primer-size-vs-target-size curve and compare it
  against `INTENT.md`'s "Known constraint" bound — that comparison is the
  actual research result this whole prototype exists to produce.
- Determine, empirically, how close *real* target data (not synthetic random
  bytes) can get to the reachable set. This needs representative example
  target files, not just random byte strings, to mean anything.

## Session logs

Chronological, narrative records of individual working sessions live as
`SESSION_<date>.md` files at the repo root — the raw notes. This document is
the standing synthesis of what they've found; keep the two in sync; don't let
this document drift stale relative to what the logs actually show.
