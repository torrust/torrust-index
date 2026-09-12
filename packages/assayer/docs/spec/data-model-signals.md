### Chapter (Host Signals) · `chap:spec:host-signals`

The chapter specifies the one block of the feature vector the host fills directly. Everything else the Core sees is derived: from a Sentinel's batch report, from a key space's competitive structure, or from products of the two. Host signals are the channel for what the host already knows and no Sentinel can observe — whether the request authenticated, how old the account is, what tier the customer sits in, what hour of the day it is. The chapter states what a declaration carries, how much of the vector each shape occupies, and when the schema is fixed.

**Schema (Signal declaration)** · `schema:signal:shapes`

A signal is declared with a name, a shape, and a persistence mode. The shape says how the host's value becomes features; the persistence mode says where the value is read from at assessment time.

| Shape | The host supplies | Encoded as |
| --- | --- | --- |
| Scalar | A value and a clipping interval | The value, clipped to the interval |
| Log-scaled | A value and a divisor | The signed logarithm of one plus the scaled magnitude |
| Binary | A truth value | One or zero, thresholded at one half |
| Ordinal | A value and its maximum | The value normalised onto the unit interval |
| Cyclic | A value and its period | The sine and cosine of the phase |
| Hashed categorical | A category and a bin count | An indicator at the bin the category hashes to |
| Vector | A fixed-length vector and a clipping interval | Each element, clipped |

Two persistence modes. An entity-persistent signal is a fact about the entity rather than about the request, and is read on the entity key from the signal cache (`tab:keyspace:signal-cache`), which is why its availability does not depend on the entity holding competitive standing anywhere. A request-scoped signal arrives with the assessment and needs no storage at all. A signal absent at assessment time is zero-filled under either mode, so a declaration is a promise about shape and never a promise about presence.

**Remark (The shapes occupy different widths)** · `rem:signal:shape-widths`

Four of the seven shapes occupy one position, and three do not. The width rule is stated here because a reader who has only the shape names will assume seven signals cost seven positions, and will be wrong by one for every cyclic signal declared.

| Shape | Width |
| --- | --- |
| Scalar, log-scaled, binary, ordinal | 1 |
| Cyclic | 2 |
| Hashed categorical | The declared bin count |
| Vector | The declared length |

The signal block's width is the sum of the declared widths, in declaration order. The cyclic shape is the one that surprises: a phase cannot be carried by a single feature without a discontinuity at the wrap point, where the last moment of one cycle and the first of the next would sit at opposite ends of the range while denoting adjacent instants, so the sine and cosine are carried together and the model learns a weight on each.

**Requirement (The signal schema is fixed at construction)** · `req:signal:schema-fixed`

The schema is declared once, at construction, and never changes. The signal block's width is fixed by that declaration, and each signal's position is derived by composing the block's start with the signal's declaration order — the schema is the authority for the order, and no position is stored per signal.

This is the block that stands still. Sentinel slots are created and destroyed with their Sentinels, per-dimension blocks widen and narrow with the outcome axis registry, and competitive indicators come and go with the competitive set; the signal block does none of these things, and its features keep their meaning for the life of the deployment. The cost is that a signal not foreseen at construction cannot be added later, and the benefit is that the one block whose semantics are entirely the host's own is also the one block no lifecycle event can move underneath it.
