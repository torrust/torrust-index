# Constants · `rec:constants:pinning-programs`

This record states the five pinning programs the package's constants cite. A constant declaration here carries a doc comment naming the identity it declares, the program that derives that identity's pin, and the pin itself. The program is what says which values the constant may hold and how its pin is spelled, so citing one is a check on the value's shape as much as a choice of derivation: a value the cited program refuses is a failure at the declaration rather than a mismatch discovered later.

The programs are not this package's invention. ADR-T-018, the constant label profile, is the doctrine they descend from; it states nine programs, of which the five below are those this crate's constants use, and it owns the argument for pinning a constant at its declaration rather than in a register. What stands here is what a constant's doc comment needs in order to be read, in the vocabulary the package's own sources cite.

Two conventions run through every program that reads a value. Underscores written as digit separators are removed before anything else, because a separator is a reading aid the value does not carry. A literal carrying a type suffix is refused by every program that reads literals, because the declaration already states the type and a suffix would put one fact in two places where only one of them is read.

**Algorithm (Count)** · `alg:const:count`

Accepts a non-negative integer literal written in decimal digits, with optional separators, no sign and no suffix. The slug is the digits with the separators removed, so `16` derives `16` and `1_000` derives `1000`.

A literal written in another base is no count and belongs to the form program. A base is a presentation of a number, and a slug that erased it would collide two spellings of one value into one name.

**Algorithm (Version)** · `alg:const:version`

Accepts what the count program accepts and derives what it derives.

The two are distinguished because the citation records intent rather than shape. A version constant is expected to be bumped, so its pin is expected to re-mint, and the citations that dangle when it does are the migration checklist rather than an accident. A reader meeting this program at a declaration is told that the churn is the design.

**Algorithm (Scalar)** · `alg:const:scalar`

Accepts a decimal floating literal: an optional leading minus, digits, optionally a decimal point and further digits, and optionally an exponent written with `e` or `E`, an optional sign, and digits. Separators are admitted and removed; a type suffix is refused.

The slug rewrites the three characters the grammar of names does not admit, and nothing else. A leading minus becomes `neg`, the decimal point becomes `p`, the exponent's letter is lowercased, an exponent's minus becomes `n` immediately after that letter, and an exponent's plus is dropped. So `0.9995` derives `0p9995`, `1e-14` derives `1en14`, `-4.0` derives `neg4p0`, and `8_760.0` derives `8760p0`. The rewriting is injective on the literals the program accepts, which an ordinary name-slugifier is not: that transformation drops a sign silently and cannot tell a decimal point from a hyphen.

**Algorithm (Seconds)** · `alg:const:seconds`

Accepts either an integer literal in the count program's shape, or a call whose callee path ends in `from_secs` and whose single argument is such a literal. The slug is that integer's digits with separators removed, so both `3600` and a second-constructor call over `3600` derive `3600`.

The program exists because a duration written as a constructor is the same fact as the number inside it, and a pin that digested the constructor's source text would change when the type was renamed.

**Algorithm (Form)** · `alg:const:form`

Accepts any value expression, and is the program the others narrow: an array or slice literal, a struct or constructor call, an arithmetic expression, a macro invocation, and equally a literal whose author wants it pinned as written rather than as read. The slug is a digest word taken over the value's normalised source text, where the value's source text is what stands between the declaration's `=` and its `;`, and the normalisation replaces every maximal run of whitespace with one space and trims the ends. A slice therefore derives one word whatever line breaks the formatter puts in it.

Two properties of this program are worth stating because a reader will otherwise assume the opposite of each. The source text is the value's and never the declaration's type, so a pin over a collection says nothing about what the collection holds. And a value that reaches out of the file is pinned by the expression that reaches rather than by what it reaches, so changing what it reaches re-mints nothing.
