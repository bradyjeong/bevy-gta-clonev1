#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldEntityType {
    Building,
    Vegetation,
    Detail,
    Landmark,
}

pub fn should_spawn_at_lod(lod_level: usize, entity_type: &WorldEntityType) -> bool {
    match entity_type {
        WorldEntityType::Vegetation => lod_level <= 1,
        WorldEntityType::Detail => lod_level == 0,
        WorldEntityType::Building => true,
        WorldEntityType::Landmark => lod_level <= 2,
    }
}
