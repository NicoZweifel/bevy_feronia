use crate::prelude::*;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;

pub fn on_add_scatter_item(
    trigger: On<Add, ScatterItem>,
    mut cmd: Commands,
    q_item: Query<(&ChildOf, Option<&ScatterLayer>), With<ScatterItem>>,
) {
    let Ok((parent, layer)) = q_item.get(trigger.target()) else {
        warn!(
            "Could not get ScatterItemLayer for ScatterItem {}",
            trigger.target()
        );
        return;
    };

    if layer.is_some() {
        return;
    };

    cmd.entity(trigger.target())
        .insert(ScatterItemOf(parent.get()));
}
