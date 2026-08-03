# SPEC.md

Status: hypothetical / pre-research. This document exists to pin down terminology and
architecture before implementation begins. Sections marked **TBD** are open questions,
not yet decided.

## Goal

Reconstruct a target file by driving a cluster of Rust servers with a per-request
"primer," rather than storing or transmitting the file's data directly. The servers
hold large precomputed state (inference pads) that, combined with a primer, is
expected to reproduce the requested piece of the target file one character at a time.

## Motivating scenario

Two machines sit on opposite ends of an extremely low-bandwidth tunnel.

- **Sender side**: holds the target data, plus a full local instance of the
  generation stack — the same defined datasets, the same generation algorithms,
  and the same "reverse" Rust/TensorFlow stack the receiver's cluster runs.
- **Receiver side**: the server cluster, holding an identical copy of the defined
  datasets/algorithms and inference pads.

**Core use case: interplanetary/interstellar links.** The scenario this is built
for is two supercomputers at opposite ends of a link spanning astronomical
distances — bandwidth is scarce and round-trip latency can be minutes, hours, or
longer, so re-requesting or negotiating mid-transfer is impractical. Both ends
must already hold everything they need (defined datasets, algorithms, pads)
before the link is used; the only thing that ever crosses it is the client file.
This is *why* provisioning has to happen out-of-band ahead of time (see below) —
over a link like this, "just ask the other side for more data" isn't a fallback
you get to have.

Both sides must be provisioned identically (datasets, algorithms, pads) *before*
the tunnel is used for anything — that provisioning happens out-of-band, since the
tunnel itself is too narrow to carry it. Once provisioned:

1. The sender performs primer discovery **locally** (no tunnel traffic): for each
   piece of the target data, it searches its own copy of the stack for a
   {dataset, coordinate, primer} that reproduces that piece.
2. The sender assembles the discovered tuples into a client file (see below) — the
   only thing that crosses the tunnel.
3. The receiver's cluster replays the client file's entries against its own local
   copy of the datasets/stack, regenerating each piece and reassembling them into
   the full data.

The entire point of the project is this asymmetry: an expensive, local, offline
search on the sender side in exchange for a tiny transmitted payload.

## Terminology

- **Defined dataset** — a dataset specified by a generation algorithm (+
  parameters/seed), not a raw blob that has to be transmitted or stored whole.
  Both sender and receiver produce identical copies by running the same algorithm
  locally. There may be more than one defined dataset in play at once. Algorithm
  details, parameters, and how many datasets exist: TBD.
- **Global dataset** — the union of all defined datasets, treated as one
  addressable space that's globally available/shared (all servers, and the
  client, reference the same underlying data). Whether it's addressed as one flat
  space or per-dataset: TBD (see Open questions).
- **Coordinate** — an address/offset into the global dataset marking where a
  generation run starts. Each server owns a coordinate range/region it's
  responsible for.
- **Inference pad** — a large data matrix maintained by an individual server,
  derived from its region of the global dataset. Created at deployment time (fixed
  for now, not yet updated at runtime). Drives the server's TensorFlow-based
  inference step.
- **Primer** — additional input sent alongside a coordinate to a server to
  condition a generation run. Format, size, and structure: **not yet defined**.
- **Server** — a single Rust executable. Owns a coordinate range, holds the
  corresponding inference pad(s) in memory, and exposes the char-in/char-out
  inference loop.
- **Cluster** — a small set of these servers, together spanning the full
  coordinate space, addressable as a group.
- **Exchange** — one round: client sends one character, server returns one
  character, generated via the TensorFlow network conditioned on the pad,
  coordinate, and the primer/prior exchanges.
- **Client file** — the artifact the sender builds and transmits: an ordered
  sequence of {dataset, coordinate, primer} entries, one (or more) per piece of
  the target file. This is the actual payload that crosses the low-bandwidth
  tunnel; everything else needed to expand it back into the original data is
  already present on both ends. Exact structure/encoding: TBD.

## Architecture (current understanding)

**Provisioning (out-of-band, one-time, not tunnel-constrained)**

1. The defined datasets exist as algorithms; both sender and receiver-side cluster
   generate identical copies locally.
2. Each server in the receiver cluster owns a coordinate range into the global
   dataset and derives its inference pad(s) from that range at deploy time. The
   sender holds the equivalent full stack locally (see Motivating scenario).

**Discovery (sender side, local, no tunnel traffic)**

3. For each piece of the target file, the sender searches its local copy of the
   stack for a {dataset, coordinate, primer} that reproduces that piece, and
   appends the result to its client file.

**Transmission (the only step that crosses the tunnel)**

4. The sender sends the client file to the receiver.

**Regeneration (receiver side)**

5. For each {dataset, coordinate, primer} entry, the receiver determines the
   owning server and sends it {coordinate, primer}.
6. The server begins inference at that coordinate: for each character the
   receiver sends, the TF network (conditioned on the pad, coordinate, and primer)
   emits one character back.
7. The receiver continues the exchange until the piece has been produced, then
   repeats for the next entry.
8. The receiver reassembles the returned pieces, in order, into the reconstructed
   file.

Note: step 3 (discovery) is the expensive, unsolved half of the project — see
"Primer discovery" below. Steps 5-8 (the runtime exchange) are comparatively
mechanical once discovery works.

## Immediate next step: bring-up + search prototype

Client file encoding can't be defined on paper — it depends on empirical results
we don't have yet. The prerequisite work, in order:

1. **Raise a reproducible stack.** Build one instance of the full stack (a defined
   dataset + its generation algorithm, pad generation, and a Rust server) that can
   be stood up identically by anyone, anywhere — this is what makes later results
   trustworthy/repeatable rather than a one-off on a single machine.
2. **Search against it.** Drive that stack with random/systematic
   {coordinate, primer} trials and observe the output: how sensitive output is to
   small primer changes, whether nearby coordinates/primers produce related
   output, how large a primer has to get before it reliably reproduces a target
   byte sequence of a given length, etc.
3. **Derive the encoding from what's found.** Once we know empirically what a
   {dataset, coordinate, primer} tuple needs to contain to be useful, the client
   file format falls out of that — not the other way around.

This reprioritizes "Primer discovery" and "Client file format" below: they're
blocked on step 1-2, not on further up-front design.

## Open questions (TBD)

- **Primer format**: what a primer is made of (length, encoding, whether it's fixed
  or grows per exchange), and how it conditions the TF network.
- **Primer discovery**: how a primer that reproduces a *specific* target chunk is
  found — differentiable search (if the TF graph is end-to-end differentiable) vs.
  black-box/discrete search (genetic algorithms, MCMC, brute force). This is
  expected to be the hardest open problem in the project.
- **Pad generation**: exact algorithm for turning random pool data into an inference
  pad, and whether pads are ever regenerated/updated post-deployment.
- **TF network architecture**: model shape, input/output character encoding,
  statefulness across exchanges within a single primer session.
- **Feasibility bound**: the whole scheme only pays off if {coordinate + primer}
  stays small relative to the target piece it reproduces. Against a *purely
  random* global dataset, finding a coordinate where a target byte sequence
  happens to occur costs roughly one bit of coordinate per bit of target
  (needle-in-haystack search) — i.e. no better than storing the data directly.
  Any real win has to come from the TF network doing genuine learned
  generation/compression (exploiting structure in the target file) rather than
  blind lookup into randomness. This needs to be worked out — probably validated
  on a small case — before scaling the cluster or the global dataset size.
- **Cluster coordination**: how work is split across servers for one target file,
  and how results are reassembled in order.
- **Dataset addressing**: whether coordinates are a single flat space spanning all
  defined datasets, or scoped per-dataset (requiring a dataset id alongside the
  coordinate in every client file entry).
- **Initial provisioning**: how sender and receiver end up with identical defined
  datasets, algorithms, and pads *before* the tunnel is used — this setup is
  assumed to happen out-of-band, but the mechanism (and how it's kept in sync if
  either side's stack changes) isn't defined yet.
- **Client file format**: encoding of the {dataset, coordinate, primer} sequence,
  and whether its own size stays negligible relative to the reconstructed data
  once real target files are tried (this is the practical, measurable version of
  the "Feasibility bound" question above). **Blocked** on the bring-up + search
  prototype above — not decidable on paper.

## Non-goals (for now)

- Runtime pad updates/mutation — pads are static post-deployment until this is
  revisited.
- Production deployment concerns (auth, networking hardening, scaling) — this spec
  covers the research prototype only.
- Keeping sender/receiver stacks in sync after initial provisioning if either side
  changes its defined datasets or algorithms — out of scope until initial
  provisioning itself is defined.
