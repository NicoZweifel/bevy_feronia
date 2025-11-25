use crate::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::relationship::Relationship;

#[cfg(feature = "tracing")]
use tracing::warn;

pub fn on_add_scatter_item(
    trigger: On<Add, ScatterItem>,
    mut cmd: Commands,
    q_item: Query<(&ChildOf, Option<&ScatterLayer>, Option<&ScatterItemOf>), With<ScatterItem>>,
) {
    let Ok((parent, layer, scatter_item_of)) = q_item.get(trigger.entity) else {
        #[cfg(feature = "tracing")]
        warn!(
            "Could not get ScatterItemLayer for ScatterItem {}",
            trigger.entity
        );
        return;
    };

    if layer.is_some() || scatter_item_of.is_some() {
        return;
    };

    cmd.entity(trigger.entity)
        .insert(ScatterItemOf(parent.get()));
}
