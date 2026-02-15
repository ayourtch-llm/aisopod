Please review the docs/plans/conversion-master-plan.md, docs/layout-legacy-openclaw.md, and docs/issues/README.md,
and then for each docs/plans/XXXX-impl-*.md plan, split it into issue tickets as per process,
for subsequent implementation by a beginner coder model.

Make the issues ordered such that every additional issue could be implemented on top of previous ones,
compiled and tested, without requiring any of the higher-numbered issues. The final implementation
of all the issue tickets should have the functionality that is equivalent to that of the codebase
in the tmp/openclaw/* codebase. If you have the capability to launch subagent per impl plan,
that may reduce the complexity for planning. Feel free to use docs/subsystems/*.md directory for creating
documentation for subsystems, as you crystallize them.

NOTE: not all of the original design patterns in the code will map cleanly to Rust, think of replacements if necessary.

Your final result from this run should be a set of issue tickets in docs/issues/open/, which, when implemented,
should result in an application that implements the functionality identical to that of the tmp/openclaw/ directory.
