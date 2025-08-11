ARCHITECTURAL REVIEW SUMMARY
The code base drifts in several places from the two core rules spelled-out in AGENT.md:

A. Simplicity First – keep modules small, single-purpose, easy to trace.
B. Event-Driven Architecture – no direct system-to-system communication across plugin boundaries; coordination must happen through light-weight events or observers.

The most acute violations are concentrated in three areas:

World/Content generation (unified_world.rs + dynamic_content.rs + entity_factory_unified.rs)
Thread-local / global state (CONTENT_RNG, placement caches, entity limits)
Cross-plugin direct calls (is_on_road_spline, RoadNetwork, UnifiedDistanceCalculator, etc.)
Below is a PRIORITISED PLAN that both pin-points the violations and prescribes concrete remediation steps.

P0 – CRITICAL FIXES (blockers for further refactor)
Adopt a Guaranteed Compile-time Anti-Pattern Gate
• Add a #[deny(unsafe_code)], #[deny(clippy::expect_used)] and a custom lint pass (or CI grep) that rejects thread_local!, std::cell::RefCell and use crate::systems::<other_plugin>:: imports outside the local plugin.
• This forces new code to follow AGENT.md going forward.

Eliminate Thread-Local Global State (violates Simplicity + Event rules)
File: dynamic_content.rs (lines 12-15)
Problem: thread_local! { static CONTENT_RNG … } hides mutable state and is not visible to the ECS schedule.
Fix:
a) Create #[derive(Resource, Default)] pub struct GlobalRng(rand::rngs::SmallRng);
b) Initialise once in App::new().insert_resource(GlobalRng::default()).
c) Inject &mut GlobalRng into systems that need randomness.
d) Delete the thread_local block.

Remove Direct System-to-System Coupling in World Generation
Symptoms:
• dynamic_content_system directly calls is_on_road_spline (roads plugin)
• entity_factory_unified.rs imports crate::systems::{RoadNetwork, is_on_road_spline, UnifiedCullable, …}
• unified_world_streaming_system manipulates Commands for spawn/unload and owns a huge UnifiedWorldManager resource
Effects: tight coupling, impossible to alter one plugin without re-compiling all, no event trace.

Immediate Containment Strategy (≤2 days):
a) Define domain events in a new events/world/ module

pub struct RequestChunkLoad { pub coord: ChunkCoord }
pub struct ChunkLoaded { pub coord: ChunkCoord }
pub struct RequestDynamicSpawn { pub pos: Vec3, pub kind: ContentType }
b) Replace calls:
• dynamic_content_system should ev_writer.send(RequestDynamicSpawn { … }) instead of calling the factory.
• unified_world_streaming_system should ev_writer.send(RequestChunkLoad { coord }) instead of spawning directly.
c) Introduce thin “executor” systems inside the same plugin that consume those events and perform the actual spawn/commands. Because executor lives in the same plugin, direct access is now legal; external callers stay decoupled.
Shrink Monolithic UnifiedEntityFactory (≈3000 LOC total)
• Split per concern following AGENT.md “one module per event group”:

factories/buildings_factory.rs
factories/vehicle_factory.rs
factories/npc_factory.rs
• Keep only light, stateless helpers in each; configuration/limits remain a Resource (EntityLimitManager) injected where needed.
P1 – HIGH-IMPACT IMPROVEMENTS (after P0, ≤1 sprint)
Turn UnifiedWorldManager into Multiple Focused Resources
a) ChunkTracker – purely chunk state & LOD.
b) PlacementGrid (already separate) – leave as is.
c) RoadNetwork already owns road data; stop re-storing inside UnifiedWorldManager (duplication).
This brings the central struct below the 10-field guideline.

Convert “spawn-on-demand” logic to Observer Pattern
• Instead of dynamic_content_system looping every few seconds, use Bevy 0.16’s Trigger<OnAdd, ActiveEntity> to react when player enters a new chunk:
fn on_chunk_loaded(trig: Trigger<OnAdd, ChunkLoaded>, …)
• Removes manual timers, makes flow explicit in the schedule.

Replace RefCell / Interior Mutability Caches
• position_cache and entity_limits contain mutable HashMaps inside a Resource – good.
• Eliminate any hidden RefCell (search for RefCell<…>).
• Use ResMut instead, every access will now be visible to Bevy borrow-checker and frame debugger.

P2 – MEDIUM PRIORITY CLEAN-UP (continuous)
Simplify Spawn Validation Logic
• is_spawn_position_valid, has_content_collision, water checks, road checks → move to a separate spawn_validation plugin that only exports pure stateless functions.
• Other plugins call it directly (allowed, as it is a utility module per AGENT.md §42 line 52).

Audit Component & Resource Sizes
• Several structs (ChunkData, UnifiedWorldManager) exceed the 64-byte/10-field advice.
• Apply #[component(immutable)] where appropriate (RoadSpline, etc.).
• Split big components into smaller ones if they change at different rates.

Event Instrumentation & Ordering
• Name executor systems handle_request_chunk_load, handle_request_dynamic_spawn as in AGENT.md §78.
• Add .before() / .after() relations:

handle_request_chunk_load.before(handle_request_dynamic_spawn)
• Add event count debug instrumentation behind the debug-ui feature.
CHECK-LIST FOR DONE CRITERIA
□ No thread_local! or static mut left; all randomness through GlobalRng resource
□ Every cross-plugin call replaced by an Event or observer unless listed in “Utility Exceptions” (§52)
□ Each event defined in its own module, ≤128 bytes, Copy/Clone.
□ No struct >10 fields unless explicitly justified; big Resources split.
□ Unit & integration tests updated (App::new().add_plugins(MinimalPlugins)…) to drive new event flow.

SEQUENCING & OWNER SUGGESTION
Week 1 (P0)
Dev A – Thread-local removal & GlobalRng
Dev B – Introduce world events + refactor dynamic_content_system to emit them
Dev C – Create executor systems inside world plugin and update schedule sets
Dev D – Slice UnifiedEntityFactory into three smaller files (mechanical move, no behaviour change)

Week 2 (P1)
Dev A – Split UnifiedWorldManager
Dev B – Convert “player moved” timers to observer pattern
Dev C – Refactor interior mutability caches
Dev QA – Add compile-time lints & CI guard, update tests

Week 3+ (P2, rolling)
Team – Component size audit, spawn_validation plugin, event instrumentation

Implementing P0 alone will make event flow explicit and decouple plugins, satisfying 80 % of AGENT.md violations. P1 & P2 are incremental and can proceed without blocking new features.