# AI execution and token-cost plan

Status: proposed routing policy. Model names and product availability can change;
verify available models and billing before treating this as a permanent setting.
This document does not configure or invoke any model.

## Immediate Luna handoff

The [Q1 document](11-gamelan-sheet-quick-win.md) is the active implementation
contract. Use its six ordered task cards and starting prompt. Do not give Luna the
entire roadmap as one instruction. High or maximum reasoning is the user's option,
not a substitute for fixed behavior and verification. No setting guarantees that
an agent never makes a mistake. Review shared contracts and actual native output.

## Model responsibilities

| Work | Starting model/effort | Review |
| --- | --- | --- |
| Requirements, architecture, public contracts, milestone decomposition | GPT-6.1 Sol / high | Product and domain review |
| New timing/pitch/realization semantics and audio concurrency | GPT-6.1 Sol / high | Independent checks and contract review |
| Scoped implementation against settled interfaces | GPT-6 Luna / medium | Behavioral checks; Sol at the appropriate gate |
| Simple documentation, forms, fixture wiring, bounded UI tasks | GPT-6 Luna / low or medium | Relevant checks and focused diff review |
| Shared-core or persistence-contract review | GPT-6.1 Sol / medium or high | Review every such change |
| Routine milestone integration review | GPT-6.1 Sol / medium | Review the milestone packet |
| A genuinely unresolved demanding problem | GPT-6 Astra selectively | Evidence-based escalation, not a default |

These are project recommendations, not a guarantee of equal model performance.
The [official model catalog](https://developers.openai.com/api/docs/models) describes
Sol as balancing intelligence/cost and Luna as economical for focused workloads.
[Official selection guidance](https://developers.openai.com/api/docs/guides/model-selection)
recommends experimenting with the same inputs and retaining the lightest model
that meets the quality requirement. Recheck these sources when changing policy.

API token rates do not automatically describe a ChatGPT/Codex subscription's usage
allowance. No exact dollar saving or fixed project token total is asserted here.

## Planning-to-execution workflow

1. Sol translates an accepted requirement into one bounded task using
   [the template](../templates/task.md). Fix expected behavior before implementation.
2. The task identifies relevant documents, existing code, allowed ownership,
   interfaces, non-goals, acceptance cases, and exact checks once available.
3. Luna reads only that context, inspects current files, and implements the task.
4. Luna runs the agreed checks and provides a compact evidence report with
   limitations. It must not redefine semantics to make its own tests pass.
5. A reviewer checks behavior and contracts, not merely whether tests are green.
6. Update the task/milestone status with evidence, then take the next dependency.

One task usually changes a small coherent behavior across a few files. File count
is a guide, not a correctness rule. Do not split a necessary transaction into
broken partial changes to satisfy an arbitrary line/file limit.

## Task contract quality

The planner must supply concrete examples, edge cases, and an unambiguous expected
result. "Build the neutral music engine" is not a Luna implementation task.
"Implement rational comparison, rejecting cross-domain comparison unless an
explicit mapping exists" can be, after bounds and mappings are specified.

Include at least one invalid input and one preservation/regression case for
shared-domain tasks. Provide independent expected values rather than deriving
them with the implementation. If correct behavior depends on a domain decision,
settle that decision before handing the task to a cheaper model.

## Review and escalation

Review every change to document meaning, schemas, time/pitch interpretation,
transaction semantics, migration behavior, or callback ownership. Routine UI/data
wiring can be grouped for milestone review once its interface is stable.

Escalate with the task contract, failing case, relevant diff, checks, and concise
root-cause hypothesis when:

- The contract conflicts with a real use case or needs a public-interface change.
- The same root problem survives two focused repair attempts.
- A task uncovers concurrency/unsafe code, irreversible data loss, or a format
  compatibility decision beyond its scope.
- Passing tests do not explain a musician-reported or native-runtime discrepancy.

The repair-attempt count is a starting anti-loop policy, not a reason to abandon
easy independent work. Do not repeatedly raise reasoning effort without improving
the evidence or narrowing the problem.

## Token efficiency

- Keep stable decisions in short documents and link to specialist details.
- Start new implementation sessions with one task and relevant file paths. Do not
  replay the entire architecture conversation or include all planning documents.
- Use specific CodeGraph queries if an index exists; otherwise narrow `rg` and
  focused reads. Indexing is optional and needs a separate decision.
- Use RTK and Context Mode when installed/available to reduce large tool outputs.
  Preserve the error/evidence needed to solve the task; truncation alone is not success.
- Reuse already gathered evidence. Rerun checks after relevant changes, not merely
  because a model or session changed.
- Avoid speculative scaffolding and duplicated planner/worker exploration.
- Keep handoffs factual: requirement IDs, changed contracts/files, checks/results,
  limitations, next task. Store large artifacts outside the conversation.

Before compaction/session switching, capture current task, status, modified files,
decisions, last verified commands/results, blockers, and next action in a concise
task note. Repository notes are the source of handoff context, not model recollection.

## Pilot before trusting the routing

After M1 interfaces are fixed, trial Luna on three tasks: a bounded validation
rule, serialization round trip, and a frontend property control when M3 starts.
Sol reviews them against independent cases. Record acceptance rate, regressions,
repair attempts, and review cost. If these tasks fail repeatedly, improve the
contracts or use Sol for that class of work.

Measure cost per accepted task, including planning, input/history/tool outputs,
reasoning/output, repair, and review. Track model/effort, elapsed time, actual usage
when available, first-pass acceptance, regressions, and human review effort. Do
not invent token totals or claim a cheaper token price implies cheaper delivery.

There is no automatic guarantee that Luna achieves Sol's judgment. The same
acceptance standard is enforced by the contract, fixtures, tools, review, and
escalation policy. Engraving/music expertise still needs human validation.

## Example execution prompt

```text
Implement task M1-03 from docs/09-first-tasks.md.
Read AGENTS.md and only the specialist documents named by that task.
Inspect current code before editing and preserve unrelated changes.
Implement its agreed interface and independent acceptance cases.
Run the relevant checks that actually exist in this repository.
Report changed files, requirement IDs, test evidence, and limitations.
If musical semantics are unresolved, identify the exact decision instead of
inventing a permanent interpretation. Continue unaffected work.
```

This prompt becomes ready only after M0 contracts and task-specific interfaces
are settled. Do not use it as permission to skip prerequisite milestones.
