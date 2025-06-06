use bevy::prelude::*;

/// Bundle for entities that need to be visible and inherit visibility from parents
#[derive(Bundle)]
pub struct VisibleBundle {
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl Default for VisibleBundle {
    fn default() -> Self {
        Self {
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
        }
    }
}

/// Bundle for child entities that inherit visibility from parents
#[derive(Bundle)]
pub struct VisibleChildBundle {
    pub inherited_visibility: InheritedVisibility,
}

impl Default for VisibleChildBundle {
    fn default() -> Self {
        Self {
            inherited_visibility: InheritedVisibility::VISIBLE,
        }
    }
}

/// Bundle for vehicle parent entities
#[derive(Bundle)]
pub struct VehicleVisibilityBundle {
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl Default for VehicleVisibilityBundle {
    fn default() -> Self {
        Self {
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
        }
    }
}
