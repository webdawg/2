# SPEC.md

Status: early research prototype. See `INTENT.md` for why this project exists —
the motivating thought experiment and the one hard constraint the design has to
respect — and `COGNITION.md` for the research methodology and what's been
empirically learned so far running the prototype. This document covers the
*how*: terminology, architecture, and the open technical questions. Sections
marked **TBD** are open, not yet decided.

## Terminology

- **Defined dataset** — a dataset specified by a generation process (a
  closed-form formula, a constant expansion like π, a seeded algorithm, or —
  potentially, later — a trained model), not a raw blob that has to be
  transmitted or stored whole. Both sender and receiver produce identical
  copies by running the same process locally, or compute pieces of it on
  demand rather than storing it at all. There may be more than one defined
  dataset in play at once. How many datasets exist and which generation
  processes back them: TBD (today: three closed-form generators, see
  `server/src/algorithms/`).
- **Global dataset** — the union of all defined datasets, treated as one
  addressable space that's globally available/shared (all servers, and the
  client, reference the same underlying data). Whether it's addressed as one flat
  space or per-dataset: TBD (see Open questions).
- **Coordinate** — an address/offset into the global dataset marking where a
  generation run starts. Each server owns a coordinate range/region it's
  responsible for.
- **Inference pad** — an optional, per-server precomputed structure derived
  from a server's region of the global dataset, used only by dataset types
  whose generation process needs precomputed state to run (e.g. a trained
  model's weights). Closed-form generators (today's three) don't need one —
  they compute directly from `{coordinate, primer}` with no precomputed pad at
  all. Whether/when a pad-requiring (e.g. learned) dataset type gets added:
  TBD, see Open questions.
- **Primer** — additional input sent alongside a coordinate to a server to
  condition a generation run. For closed-form generators (today) this is just
  the formula's parameters — e.g. an LCG's seed/multiplier/increment, or a
  polynomial's `a, b, c` — carried in the protocol's `params` field. Whether a
  future dataset type needs a richer primer format: TBD.
- **Server** — a single Rust executable. Owns a coordinate range and exposes a
  request/response interface that answers `{coordinate, length, primer}`
  queries within that range — today by direct closed-form computation; a
  pad-backed inference step is one possible future extension, not a
  requirement.
- **Cluster** — a small set of these servers, together spanning the full
  coordinate space, addressable as a group.
- **Exchange** — one request/response round trip: the client sends
  `{dataset, coordinate, length, primer}`, the server computes and returns the
  requested bytes in that one response (see `server/src/protocol.rs`). Earlier
  drafts of this spec described a per-character loop driven by a TensorFlow
  network instead; that's been set aside for now in favor of the simpler
  closed-form path — see `COGNITION.md` for why.
- **Client file** — the artifact the sender builds and transmits: an ordered
  sequence of {dataset, coordinate, primer} entries, one (or more) per piece of
  the target file. This is the actual payload that crosses the low-bandwidth
  tunnel; everything else needed to expand it back into the original data is
  already present on both ends. Exact structure/encoding: TBD.

## Architecture (current understanding)

**Provisioning (out-of-band, one-time, not tunnel-constrained)**

1. The defined datasets exist as generation processes (today: closed-form
   algorithms); both sender and receiver-side cluster produce identical output
   locally, computed on demand rather than stored. (If a pad-requiring dataset
   type is added later, this is also where each server would derive its
   inference pad from its coordinate range — see Terminology.)
2. Each server in the receiver cluster owns a coordinate range into the global
   dataset. The sender holds the equivalent full stack locally (see
   `INTENT.md`'s Motivating scenario).

**Discovery (sender side, local, no tunnel traffic)**

3. For each piece of the target file, the sender searches its local copy of the
   stack for a {dataset, coordinate, primer} that reproduces that piece, and
   appends the result to its client file. Today: brute-force random search
   (`client/`) — see `COGNITION.md` for what that's found so far and why it's a
   baseline/mechanics-validation tool rather than the eventual method.

**Transmission (the only step that crosses the tunnel)**

4. The sender sends the client file to the receiver.

**Regeneration (receiver side)**

5. For each {dataset, coordinate, primer} entry, the receiver determines the
   owning server and sends it {coordinate, length, primer}.
6. The server computes the requested range directly from the dataset's
   generation process, conditioned by the primer, and returns the bytes in one
   response (see `server/src/protocol.rs`, `server/src/algorithms/`).
7. The receiver reassembles the returned pieces, in order, into the
   reconstructed file.

Note: step 3 (discovery) is the expensive, unsolved half of the project — see
"Primer discovery" below and `COGNITION.md` for the research approach being
used to attack it. Steps 5-7 (the runtime exchange) are comparatively
mechanical once discovery works, and are already implemented for the
closed-form case (see `server/`).

## Research methodology

Client file encoding, and the primer discovery method itself, can't be settled
on paper — they depend on empirical results. This project runs on a
build-search-learn loop rather than up-front design; see `COGNITION.md` for the
loop itself, the current substrate (why closed-form generators are being used
as a stepping stone rather than the final "physical constants" repositories
from `INTENT.md`), and what's been found running it so far.

This reprioritizes "Primer discovery" and "Client file format" below: they're
blocked on that loop, not on further up-front design.

## Open questions (TBD)

- **Primer format**: what a primer is made of (length, encoding, whether it's
  fixed or grows per exchange). For closed-form generators (today) it's just
  the formula's parameters (see `server/src/algorithms/`); still open is
  whether a richer/variable-length format is needed once real target data is
  tried.
- **Primer discovery**: how a primer that reproduces a *specific* target chunk
  is found. Brute-force random search (current, see `COGNITION.md`) is known
  not to scale; candidates worth exploring include closed-form parameter
  solving (e.g. algebraic recovery of a polynomial's or LCG's parameters from
  sample output, rather than guessing them) and black-box discrete search
  (genetic algorithms, MCMC). Still expected to be the hardest open problem in
  the project.
- **Learned/model-based dataset type (optional path)**: if closed-form
  generators can't reach useful primer sizes for real target data, a trained
  model is one candidate extension (this is what the original "inference pad" /
  TensorFlow framing was reaching for). If pursued, it reopens: pad generation
  (how pool data becomes a pad), network architecture, and whether/how pads
  update post-deployment. Not needed for the current closed-form research path,
  and not started.
- **Feasibility bound**: see `INTENT.md`'s "Known constraint" for the hard limit
  (primer size bounded by the target's actual information content, regardless
  of compute or repository size). What's still open is empirical: how large a
  class of real-world target data has low-enough information content relative
  to the repositories to be reachable with a primer near the 45KB motivating
  figure, and how sender-side search finds that relationship efficiently. This
  needs to be validated on small cases before scaling the cluster or the
  repository size.
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
  the "Feasibility bound" question above). **Blocked** on the research
  methodology loop above (see `COGNITION.md`) — not decidable on paper.

## Non-goals (for now)

- Runtime pad updates/mutation — pads are static post-deployment until this is
  revisited.
- Production deployment concerns (auth, networking hardening, scaling) — this spec
  covers the research prototype only.
- Keeping sender/receiver stacks in sync after initial provisioning if either side
  changes its defined datasets or algorithms — out of scope until initial
  provisioning itself is defined.
