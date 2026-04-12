# Bond Navgraph and Perception Spec

This document defines the Bond-style navigation and perception layer, inspired by N64-era STAN/PAD systems.

## Concepts

- Room (STAN): A logical partition of the level geometry. Each room has an integer ID.
- Portal (PAD): A connection between two rooms, optionally gated by door state.
- Navnode: A point in space guards can stand on or navigate through, tagged with a room ID.

## Data Model

- RoomId: u16
- NavNodeId: u32

Rooms:
- Each room has:
  - id: RoomId
  - name: optional string label
  - bounds: AABB or convex polygon (for debugging and editor use)

Portals:
- Each portal connects exactly two rooms.
- A portal may be open or closed at runtime.
- Closed portals block visibility and pathfinding unless the door is open.

Navnodes:
- Each navnode stores:
  - id: NavNodeId
  - position: Vec3
  - room_id: RoomId
  - flags: bitfield (e.g., cover spot, patrol point, alarm panel)

Room adjacency:
- Precomputed from the portal list.
- Represented as a symmetric adjacency matrix or adjacency list:
  - adjacency[room_a][room_b] = true if reachable via one portal hop.

Room visibility matrix:
- Precomputed conservative visibility:
  - visible[room_a][room_b] = true if entities in room_a can ever see entities in room_b.
- In the first pass, visible == adjacency (same or adjacent rooms).
- Later passes can include multi-hop visibility and door state.

## ECS Components

- RoomTransform:
  - position: Vec3
  - room_id: RoomId

- NavNodeRef:
  - node_id: NavNodeId

- RoomGraph (resource):
  - rooms: array of rooms
  - portals: array of portals
  - adjacency: matrix or adjacency list
  - visibility: matrix

## Queries

The navgraph exposes only a few key queries to gameplay systems:

- is_same_or_visible_room(room_a, room_b) -> bool
  - Returns true if room_a == room_b or visibility[room_a][room_b] is true.

- neighbors(room) -> iterator over adjacent rooms

- find_navnode_near(position) -> NavNodeId
  - Debug / editor use; pathfinding uses prebuilt nav data.

## Perception Simplification

Guard perception uses room-level checks before any geometric tests:

- If guard.room and player.room are not visible according to visibility matrix:
  - Guard cannot see or hear player (V_vision = 0, V_sound = 0).
- Else:
  - Perception systems compute detailed scores (visibility, sound) based on distance and other factors.
