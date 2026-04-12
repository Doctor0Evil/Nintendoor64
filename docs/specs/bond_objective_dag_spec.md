# Bond Objective DAG Spec

This spec defines mission objectives as a directed acyclic graph (DAG).

## Concepts

- Objective node O_i: a goal the player can complete or fail.
- Edge O_i -> O_j: O_j becomes eligible after O_i is complete.
- Mission state s: ECS-controlled world state (entities, flags, counters).

## Completion Rule

Each objective O_i has:

- Condition C_i(s): function returning true if objective is satisfied in state s.
- Predecessor set pre(O_i): all objectives that must be complete first.

An objective is complete if:

- complete(O_i, s) = C_i(s) AND for all O_j in pre(O_i), complete(O_j, s) is true.

## Failure Conditions

Optional failure predicate F_i(s) per objective.

If F_i(s) becomes true before completion, objective fails.

## Implementation

- The DAG is specified in JSON (see schema).
- A runtime system:
  - Evaluates C_i(s) for all nodes each tick.
  - Updates objective status (Pending, Active, Complete, Failed).
  - Emits mission events when status changes.
