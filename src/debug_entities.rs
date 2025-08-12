use bevy::prelude::*;

pub fn debug_entities(
    query: Query<(Entity, &Transform, Option<&Name>), With<Mesh3d>>,
) {
    let count = query.iter().count();
    if count == 0 {
        println!("DEBUG: No entities with Mesh3d found!");
    } else {
        println!("DEBUG: Found {} entities with Mesh3d", count);
        for (entity, transform, name) in query.iter().take(5) {
            let name_str = name.map(|n| n.as_str()).unwrap_or("Unnamed");
            println!(
                "  - Entity {:?} ({}): pos={:?}",
                entity, name_str, transform.translation
            );
        }
    }
}
