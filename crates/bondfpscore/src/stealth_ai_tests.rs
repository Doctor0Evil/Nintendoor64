// File: crates/bondfpscore/src/stealth_ai_tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> StealthConfig {
        StealthConfig {
            max_vision_distance: 25.0,
            vision_distance_gamma: 1.5,
            light_gamma: 1.2,
            posture_standing: 1.0,
            posture_crouched: 0.6,
            posture_prone: 0.3,
            movement_still: 0.5,
            movement_walk: 1.0,
            movement_run: 1.5,
            awareness_max: 100.0,
            awareness_decay_per_sec: 2.0,
            threshold_suspicious: 20.0,
            threshold_alert: 60.0,
            hysteresis_suspicious: 5.0,
            investigate_timeout_sec: 5.0,
        }
    }

    fn simple_rooms() -> RoomAdjacency {
        RoomAdjacency {
            adjacency: vec![
                vec![true, true],
                vec![true, true],
            ],
        }
    }

    #[test]
    fn visibility_zero_outside_fov_or_room() {
        let cfg = default_cfg();
        let rooms = simple_rooms();
        let player_sig = PlayerStealthSignature {
            posture: Posture::Standing,
            movement: Movement::Still,
            light_level: 1.0,
        };

        let guard_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let player_pos = Vec3 { x: 100.0, y: 0.0, z: 0.0 };

        let vis = GuardVision {
            forward: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
            max_distance: 25.0,
            cos_fov_half: (45.0f32.to_radians()).cos(),
            base_detectability: 1.0,
        };

        let v = compute_visibility_score(
            guard_pos,
            RoomId(0),
            &vis,
            player_pos,
            RoomId(0),
            &player_sig,
            &rooms,
            &cfg,
        );
        assert_eq!(v, 0.0);
    }

    #[test]
    fn awareness_accumulates_and_triggers_alert() {
        let cfg = default_cfg();
        let rooms = simple_rooms();
        let sounds = SoundEvents { events: Vec::new() };

        let player_xform = StealthTransform {
            position: Vec3 { x: 5.0, y: 0.0, z: 0.0 },
            room: RoomId(0),
        };
        let player_sig = PlayerStealthSignature {
            posture: Posture::Standing,
            movement: Movement::Walk,
            light_level: 1.0,
        };

        let mut guard_stealth = GuardStealth {
            state: StealthState::Patrol,
            awareness: 0.0,
            can_trigger_alarm: true,
            time_since_last_visual: 0.0,
        };
        let guard_vis = GuardVision {
            forward: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
            max_distance: 25.0,
            cos_fov_half: (60.0f32.to_radians()).cos(),
            base_detectability: 1.0,
        };
        let guard_xform = StealthTransform {
            position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            room: RoomId(0),
        };

        let mut events = StealthMissionEvents::default();

        // Simulate a few seconds of the player in plain sight.
        let mut guards: Vec<(&mut GuardStealth, &GuardVision, &StealthTransform)> =
            vec![(&mut guard_stealth, &guard_vis, &guard_xform)];

        for _ in 0..120 {
            update_stealth_system(
                1.0 / 30.0,
                &cfg,
                &rooms,
                &sounds,
                &player_xform,
                &player_sig,
                &mut guards,
                &mut events,
            );
        }

        assert!(
            guard_stealth.awareness >= cfg.threshold_alert,
            "guard should reach alert threshold"
        );
        assert!(
            matches!(guard_stealth.state, StealthState::Alert | StealthState::Attack),
            "guard should be in alert or attack state"
        );
        assert!(
            events.events.iter().any(|e| matches!(e, StealthMissionEvent::AlarmRaised)),
            "alarm should be raised at least once"
        );
    }

    #[test]
    fn deterministic_update_for_same_inputs() {
        let cfg = default_cfg();
        let rooms = simple_rooms();
        let sounds = SoundEvents { events: Vec::new() };

        let player_xform = StealthTransform {
            position: Vec3 { x: 5.0, y: 0.0, z: 0.0 },
            room: RoomId(0),
        };
        let player_sig = PlayerStealthSignature {
            posture: Posture::Standing,
            movement: Movement::Walk,
            light_level: 1.0,
        };

        let base_guard_stealth = GuardStealth {
            state: StealthState::Patrol,
            awareness: 0.0,
            can_trigger_alarm: true,
            time_since_last_visual: 0.0,
        };
        let guard_vis = GuardVision {
            forward: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
            max_distance: 25.0,
            cos_fov_half: (60.0f32.to_radians()).cos(),
            base_detectability: 1.0,
        };
        let guard_xform = StealthTransform {
            position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            room: RoomId(0),
        };

        let mut seq1 = base_guard_stealth;
        let mut seq2 = base_guard_stealth;

        let mut events1 = StealthMissionEvents::default();
        let mut events2 = StealthMissionEvents::default();

        // Run two independent simulations with identical inputs.
        let mut guards1: Vec<(&mut GuardStealth, &GuardVision, &StealthTransform)> =
            vec![(&mut seq1, &guard_vis, &guard_xform)];
        let mut guards2: Vec<(&mut GuardStealth, &GuardVision, &StealthTransform)> =
            vec![(&mut seq2, &guard_vis, &guard_xform)];

        for _ in 0..180 {
            update_stealth_system(
                1.0 / 30.0,
                &cfg,
                &rooms,
                &sounds,
                &player_xform,
                &player_sig,
                &mut guards1,
                &mut events1,
            );
            update_stealth_system(
                1.0 / 30.0,
                &cfg,
                &rooms,
                &sounds,
                &player_xform,
                &player_sig,
                &mut guards2,
                &mut events2,
            );
        }

        assert!(
            (seq1.awareness - seq2.awareness).abs() <= 0.0001,
            "awareness should be identical for deterministic simulation"
        );
        assert_eq!(seq1.state, seq2.state, "states should match");
    }
}
