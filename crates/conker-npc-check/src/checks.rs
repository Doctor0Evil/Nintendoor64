// Destination: crates/conker-npc-check/src/checks.rs

use crate::model::{CheckReport, SessionProfile};
use anyhow::Result;
use conker_schema::{ConkerMapRecipe, NpcContract};
use std::collections::{BTreeMap, BTreeSet};

pub fn run_all_checks(
    maps: &[(String, ConkerMapRecipe)],
    npc_contracts: &[(String, NpcContract)],
    session: &SessionProfile,
) -> Result<CheckReport> {
    let mut report = CheckReport::default();

    let npc_by_id: BTreeMap<String, &NpcContract> =
        npc_contracts.iter().map(|(id, npc)| (id.clone(), npc)).collect();

    // 1. Basic: ensure each NPC id is unique.
    if npc_by_id.len() != npc_contracts.len() {
        report.push("duplicate NPC IDs found in npc contracts");
    }

    // 2. Map-level: ensure any NPC referenced by maps has a contract.
    check_maps_reference_existing_npcs(maps, &npc_by_id, &mut report);

    // 3. Design-contract specific invariants.
    if session.enforce_zombie_headshot_rule {
        check_zombie_headshot_rule(&npc_by_id, &mut report);
    }

    if session.enforce_pickup_only_arsenal {
        check_no_class_or_loadout_fields(&npc_by_id, &mut report);
    }

    Ok(report)
}

fn check_maps_reference_existing_npcs(
    maps: &[(String, ConkerMapRecipe)],
    npc_by_id: &BTreeMap<String, &NpcContract>,
    report: &mut CheckReport,
) {
    // For now, assume maps have a simple npcIds field; if not, you can
    // extend ConkerMapRecipe with whatever structure you're using.
    for (map_id, _map) in maps {
        // Placeholder hook: if your map recipe carries NPC IDs, check them here.
        // Example:
        // for npc_ref in &map.npc_refs {
        //     if !npc_by_id.contains_key(&npc_ref.npc_id) {
        //         report.push(format!(
        //             "map '{}' references unknown NPC '{}'",
        //             map_id, npc_ref.npc_id
        //         ));
        //     }
        // }
        let _ = (map_id, npc_by_id); // silence unused warning until wired.
    }
}

/// Enforce that zombie contracts have headshot-style kill tags and ignore
/// bodyshot tags, matching the N64 "headshots / certain blasts only" behavior.
fn check_zombie_headshot_rule(
    npc_by_id: &BTreeMap<String, &NpcContract>,
    report: &mut CheckReport,
) {
    for (id, npc) in npc_by_id {
        if npc.kind.to_string().to_uppercase() != "ZOMBIE" && !id.contains("zombie") {
            continue;
        }

        let kill_tags: BTreeSet<_> = npc.damageable.kill_tags.iter().collect();

        let has_headshot = kill_tags
            .iter()
            .any(|tag| tag.contains("headshot") || tag.contains("HEADSHOT"));

        if !has_headshot {
            report.push(format!(
                "NPC '{}' (kind ZOMBIE) is missing a headshot kill tag in damageable.killTags",
                id
            ));
        }

        let ignore_tags: BTreeSet<_> = npc.damageable.ignore_tags.iter().collect();
        let ignores_body = ignore_tags
            .iter()
            .any(|tag| tag.contains("bodyshot") || tag.contains("BODYSHOT"));

        if !ignores_body {
            report.push(format!(
                "NPC '{}' (kind ZOMBIE) should ignore at least one bodyshot tag in damageable.ignoreTags",
                id
            ));
        }
    }
}

/// Guardrail: reject obvious attempts at class/loadout mechanics in NPC contracts.
/// This is a blunt instrument but effective as a CI fence.
fn check_no_class_or_loadout_fields(
    npc_by_id: &BTreeMap<String, &NpcContract>,
    report: &mut CheckReport,
) {
    for (id, npc) in npc_by_id {
        // Here we don't have explicit class fields in NpcContract, so we just
        // enforce that IDs and titles don't signal classes.
        let lower_id = id.to_lowercase();
        let lower_title = npc.title.to_lowercase();

        let banned_markers = [
            "class.",
            ".assault",
            ".sniper",
            ".medic",
            ".engineer",
            "loadout",
            "perk",
        ];

        if banned_markers.iter().any(|m| lower_id.contains(m) || lower_title.contains(m)) {
            report.push(format!(
                "NPC '{}' uses class/loadout-like naming ('{}'); this violates the pickup-only arsenal rule",
                id, npc.title
            ));
        }
    }
}
