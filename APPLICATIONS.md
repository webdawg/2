# APPLICATIONS.md

Status: this document is downstream of `INTENT.md`'s "Known constraint" — it
explores *what kinds of real target data* this scheme could plausibly help
with, and what it can't, given that constraint. It's exploratory/speculative,
not a roadmap or a promise: nothing here is implemented, and inclusion in this
document is not the same as being validated by `COGNITION.md`'s empirical
loop. See `INTENT.md` for why the project exists, `SPEC.md` for the
architecture, and `COGNITION.md` for what's actually been tested so far.

## The filter every candidate has to pass

`INTENT.md`'s Known constraint is the filter: a primer can only address as
much data as its own information content allows. So the only target data
worth considering here is data that has **genuine low complexity relative to
the shared repositories** — real structure, redundancy, or an actual short
generating relationship — not data that merely compresses well in isolation,
and not data that's arbitrary/high-entropy relative to everything both sides
share. Every application idea below has to be read through that filter; most
plausible-sounding candidates fail it on inspection, which is itself a useful
research output (see "Rejected or unresolved candidates").

## Candidate application areas

- **Scientific/mathematical datasets with a known closed form.** Data that is
  itself a slice of a well-known constant or sequence (digits of π, e,
  physical constants, values of a known function over a range) is close to
  the motivating scenario by construction — the "primer" is close to just
  "which slice." The open question is how much *real* scientific data
  actually looks like this versus merely being described by one after the
  fact.
- **Procedurally-generated or simulation-derived content.** Data whose
  origin already *is* a short generation process plus a seed (terrain,
  textures, particle systems, certain synthetic datasets) is a natural fit —
  if the receiving side can hold/run the same generator, the primer is close
  to the original seed and parameters, which is often already small.
- **Highly redundant archival/scientific data.** Large datasets with heavy
  internal self-similarity (long time series with periodic structure,
  repeated-pattern sensor logs) may decompose into short
  {dataset, coordinate, primer} references against shared repositories more
  effectively than generic compression, if the redundancy lines up with
  something in the repository rather than just within the file itself.
- **Extremely bandwidth/latency-constrained links.** The motivating scenario
  (interplanetary/interstellar) is the extreme case, but any link where
  bandwidth is the dominant cost and compute is comparatively free on both
  ends (deep space probes, some satellite links) is the right shape of
  problem for this approach *if* the data being sent also passes the
  complexity filter above — the constraint doesn't relax just because the
  link is expensive.
- **AI-to-AI private conversation, via a different mechanism than the rest of
  this list.** Two AI agents that pre-share the same repositories could
  exchange `{dataset, coordinate, primer}` tuples that are meaningless to
  anyone who doesn't hold the matching repository — an eavesdropper on the
  wire sees only coordinates and short parameters, not content. This is
  **not** the search-for-a-match mechanism used elsewhere in this document,
  and it is not exempt from the Known constraint either way:
  - If it's done as *search* (find a coordinate whose repository content
    already matches the message), it inherits the same complexity filter as
    every other candidate above — arbitrary/private conversational content
    is exactly the high-entropy case the constraint rules out, so this only
    compresses well for conversation content that happens to have real
    structure relative to the repositories.
  - If it's done as a *keystream* instead (combine the message with
    repository bytes at an agreed coordinate — e.g. XOR — rather than
    searching for a match), it sidesteps the search-cost problem and works
    for arbitrary content, but it is then a one-time-pad-style stream
    cipher, not a compression scheme: the transmitted primer is small, but
    confidentiality depends on standard shared-secret-stream properties, not
    on anything specific to this project. In particular, reusing a
    coordinate/keystream range for two different messages is as
    catastrophic as OTP key reuse, and the repository would need to
    function as genuine one-time key material (never reused, not derivable
    by an observer) rather than a public, independently-regenerable spec —
    which is in tension with `SPEC.md`'s current assumption that
    repositories are shared and known, not secret.
  - The privacy value proposition here (confidentiality) is distinct from
    this project's original bandwidth value proposition, and would need its
    own threat model before being taken seriously — it's recorded here as a
    candidate worth thinking through, not as a validated use case.

## Rejected or unresolved candidates

Recording these because ruling something out is as valuable as finding a fit:

- **General-purpose file/data transport (arbitrary data).** Fails the filter
  directly — this is the case `INTENT.md` already rules out. No amount of
  repository size or compute changes it for genuinely high-entropy data.
- **Already-compressed or encrypted data.** By design this data is built to
  look maximally high-entropy/incompressible; it's close to the worst case
  for this scheme, not a good candidate, even though people often want to
  transport exactly this kind of data.
- **"Any real-world file, because real files aren't truly random."** True in
  general (see algorithmic information theory) but not sufficient — the
  data's structure has to relate to *these specific shared repositories*, not
  just be non-random in some abstract sense. This gap (real structure that
  exists, vs. structure the current repositories can actually address) is
  unresolved and is one of `COGNITION.md`'s open research directions.

## Open questions

- How is a candidate target checked for "passes the filter" *before* an
  expensive discovery search is run against it, rather than discovering it
  doesn't fit only after search fails?
- Do different application areas above need different repository designs
  (e.g. a terrain-generation primer format looks nothing like a
  digits-of-π coordinate), and if so, does that reopen `SPEC.md`'s "Dataset
  addressing" open question?
- Is there real, obtainable example data for any candidate above that could
  be used to run `COGNITION.md`'s empirical loop against, instead of
  synthetic random bytes? This is the concrete next step that would move an
  entry here from "candidate" to "tested."
- For the AI-to-AI private conversation candidate: does a keystream/OTP-style
  mechanism belong in this project at all, given it needs a *secret*
  repository rather than a shared-but-public spec, which conflicts with
  `SPEC.md`'s current provisioning assumptions? Worth a real threat model
  before treating it as more than a thought experiment.
