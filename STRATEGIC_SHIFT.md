# STRATEGIC SHIFT: Oracle-Guided Architecture Change

## DECISION
**Moving from bevy_ecs 0.13 + micro-crates to Bevy 0.16.1 + strategic modularity**

## WHY
Current architecture fights Bevy ecosystem, creates unnecessary complexity:
- ❌ Reinventing RON loaders, wgpu wrappers, asset pipelines  
- ❌ Cross-crate compilation overhead dominates CI time (40%+)
- ❌ Test failures from mocked ECS instead of integrated Bevy App
- ❌ Future Bevy upgrades require multi-month re-integration

## TARGET ARCHITECTURE
```
├─ crates/
│   ├─ amp_core/          # Pure Rust utilities, error handling (no Bevy deps)
│   ├─ amp_math/          # glam re-exports, Morton, AABB (no Bevy deps)  
│   ├─ amp_engine/        # Bevy 0.16.1 dependency, engine plugins
│   ├─ amp_gameplay/      # Game systems, components, prefabs
│   └─ amp_tools/         # xtask, build pipeline helpers (optional)
```

## MIGRATION PLAN (10-14 DAYS)
1. **Days 1-2**: Branch & lock Bevy 0.16.1 versions
2. **Days 3-4**: Consolidate amp_spatial, amp_gpu, amp_world → amp_engine  
3. **Days 5-6**: Replace custom RON loader with Bevy asset pipeline
4. **Days 7-9**: Rewrite tests to use App::new().add_plugins(DefaultPlugins)
5. **Days 10-14**: Documentation, stabilization, playtest

## EXPECTED BENEFITS
- ✅ **Ecosystem Leverage**: Full Bevy plugins, examples, community support
- ✅ **Compile Performance**: 30-40% faster builds  
- ✅ **Test Reliability**: Integrated App-based testing
- ✅ **Future-Proofing**: Bevy 0.17+ upgrades = cargo upgrade + minor fixes
- ✅ **Amp Productivity**: Clear boundaries without micro-crate coordination tax

## STATUS
- ✅ Oracle consultation complete
- ✅ ADR-007 created  
- ✅ Agent.md updated
- ✅ Documentation aligned
- 🔄 Ready for implementation

**All documentation is now aligned to this strategy. Proceed with Oracle's migration plan.**
