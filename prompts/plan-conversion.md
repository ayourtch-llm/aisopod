Please study carefully the project in tmp/openclaw/ and plan its rewrite into a Rust project called "aisopod".

First make a detailed existing codebase layout and functioning document in layout document in docs/layout-legacy-openclaw.md,
which can be used later by other sessions.

Then use that document as a support base to analyze further and to write a conversion plan, in 
docs/plans/conversion-master-plan.md, which would cover all of the areas in high level, such that the full existing functionality
is covered, including the UI close enough but with a variation related to the new name. The rewrite should retain all
the functionality.

Then, from that plan create more detailed implementation plans, stored in docs/plans/XXXX-impl-name-of-plan.md, 
such that XXXX, numerically sorted, would allow one to implement the entire functionality. 

These plans should cover roughly one item from the original high level plan, each.
They will be also a source for the creating of the detailed implementation tickets later, but not yet.
