# Bond Lock-On Cone Spec

This spec defines a reusable auto-aim / lock-on cone pattern.

## Geometry

Given:

- Aim direction a (unit vector)
- Direction to target t (unit vector)
- Maximum lock-on angle θ_max
- Maximum lock-on distance d_max

A target is lock-on eligible if:

- dot(a, t) >= cos(θ_max)
- distance_to_target <= d_max

## Scoring

To choose between multiple candidates:

- Compute score = dot(a, t).
- Prefer higher score (more central target).
- Break ties by nearer distance.

## Parameters

- θ_max_degrees in [0, 90]
- d_max in engine units

These parameters are stored in a JSON file and applied per weapon or per aim profile.
