# Bond Guard State Machine

This spec defines the canonical guard behavior states and transitions for Bond-like and PD-like FPS games.

## States

- Idle: Guard is stationary, not patrolling.
- Patrol: Guard walks along a predefined path.
- Suspicious: Guard has detected weak signals (sounds, glimpses) but has not confirmed the player.
- Investigate: Guard moves toward last known or suspected position.
- Alert: Guard has confirmed the player and will attack or raise alarm.

## Continuous Awareness

- Each guard has an awareness scalar A in [0, 1].
- A(t + Δt) = clamp(A(t) + V_total Δt - k_decay Δt, 0, 1)
  - V_total is sum of vision and sound visibility scores for that tick.
  - k_decay is the decay per second.

## Thresholds

- T_suspicious: A >= T_suspicious enters Suspicious/Investigate.
- T_alert: A >= T_alert enters Alert.

Constraints:

- 0 <= T_suspicious < T_alert <= 1.

## Transitions

- Idle/Patrol:
  - If A >= T_alert: transition to Alert.
  - Else if A >= T_suspicious: transition to Suspicious.
- Suspicious:
  - If A >= T_alert: Alert.
  - If A falls below T_suspicious for a sustained period: back to Patrol.
- Investigate:
  - Similar to Suspicious, but guard moves toward last_seen position.
- Alert:
  - Remains in Alert while A >= T_suspicious or target is visible.
  - If A decays below T_suspicious and target is not visible: Investigate.

The implementation must be pure and driven only by A, thresholds, and simple flags (has_visual, last_seen_time).
