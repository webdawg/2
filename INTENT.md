# INTENT.md

Status: this is the project's *why* — the motivating thought experiment and the
one hard constraint the design has to respect. It should change rarely; when
the vision genuinely shifts, update it here first, then bring `SPEC.md` (the
*how*) and `COGNITION.md` (the research methodology/log) in line with it.

## Goal

Reconstruct a target file — of arbitrary size — by transmitting a small "primer"
across an extremely constrained channel, instead of transmitting the file's data
directly. Both ends independently hold (or can regenerate) large data
repositories from a shared spec; nothing about those repositories ever crosses
the channel, only the primer that tells the receiving side how to compute its
way to the target within its own copy.

The scale that motivates this project (see "Motivating scenario") is
reconstructing something on the order of a 2PB file from something on the order
of 45KB of primer data. That specific ratio is the thing this research is
*discovering the real boundaries of* — it is not a proven starting guarantee.
See "Known constraint" for the one hard limit this project's design has to
operate within regardless of approach, and `SPEC.md`'s Open questions for
what's still genuinely unknown.

## Motivating scenario

Two unimaginably powerful supercomputers sit at opposite ends of a galaxy. The
link between them can carry only a tiny amount of data — on the order of 45KB —
but the goal is to reproduce a target file on the order of 2PB on the far side.

Both machines share a known spec and design. Each holds, or can independently
compute, enormous data repositories representing constants carried out to
extreme precision or extent — the digits of π computed arbitrarily far out,
other physical/mathematical constants, and combinations of them read out over a
chosen span, starting point, and direction. These repositories are never
transmitted between the two machines: they are generated identically and
independently on each side from the shared spec. This is the crux of the design
— the repositories can be arbitrarily large because building them costs nothing
to transmit, only compute.

1. **Sender side**: holds the target file and an identical copy of the
   generation stack. It searches **locally** (no tunnel traffic) for a starting
   point, a direction/path through the repositories, and a primer that, computed
   out, reproduces the target file.
2. The sender assembles what it found into a client file (see `SPEC.md`'s
   Terminology) — the only thing that ever crosses the link.
3. **Receiver side**: given just the client file's {dataset, coordinate,
   primer} entries, it computes — potentially for years or decades, using its
   own enormous compute budget — until it has reproduced the exact output the
   sender found, then reassembles the pieces into the target file.

The entire point of the project is this asymmetry: an expensive, local, offline
search on the sender side (and potentially an expensive computation on the
receiver side) in exchange for a payload that stays as small as the target's
actual reproducibility allows.

**What the primer size actually depends on.** 45KB is the motivating target
figure, not a proven ceiling. How small a primer can get for a given file is
itself one of this project's central research questions (see "Known constraint"
and `SPEC.md`'s Open questions). It will vary by target: some data will be found
cheaply with a very small primer; other data may need a much larger one, or may
not be reachable within a practical primer size at all. Characterizing that
boundary — not assuming it away in either direction — is the research.

## Known constraint: primer size is bounded by the target's information content

This is a boundary the design has to operate within, not a limitation of
today's algorithms — and it does not change with more compute, cleverer search,
larger repositories, or more time on the receiving side, including hypothetical
future compute. It's worth stating plainly because it shapes where the real
research wins can come from:

A primer of a given size can only ever specify a limited number of distinct
outcomes — a primer built from N bits of information can point to at most 2^N
distinct results, whatever those results are computed from. The repositories
are fixed and identical on both sides *before* any specific transfer, agreed as
part of the shared spec — so they carry no information about *which* file is
being sent this time; only the primer does that. Consequences:

- A file's actual information content (how unpredictable it is, not its raw
  byte size) has to fit inside what the primer can address. Structured,
  redundant, or formula-derived data can have very low information content
  relative to its size — a huge span of π's digits has an information content
  of roughly "which digits, starting where," independent of how many digits it
  spans.
- Data with no such structure relative to the repositories — arbitrary,
  maximum-entropy data — cannot be addressed by a primer smaller than the data
  itself, by any method, because there is no way around comparing how many
  distinct target-sized files exist against how many distinct primers exist.
- Receive-side compute time (years, decades, unbounded) doesn't change this: more
  time changes how expensive it is to *carry out* what a primer specifies, not
  how many distinct outcomes a primer of a given size can specify in the first
  place.

This isn't a reason to abandon the project — it's the boundary that tells the
research where real wins are possible: target data with a genuine relationship
to (or low complexity relative to) the shared repositories is where this scheme
can work, potentially very well. The open research question is how large that
class of real-world data actually is, and how sender-side search finds the
relationship when one exists.
