use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryFilter;

/// Helper to find all entities at or below `item_root` matching the component query.
pub fn iter_self_and_descendants_with_component<'q, F>(
    item_root: Entity,
    q_children: &'q Query<&Children>,
    q_search_component: &'q Query<Entity, F>,
) -> impl Iterator<Item = Entity> + 'q
where
    F: QueryFilter,
{
    let self_iter = q_search_component
        .get(item_root)
        .ok()
        .map(|_| item_root)
        .into_iter();

    let children_iter = q_children
        .iter_descendants(item_root)
        .filter(move |&descendant_entity| q_search_component.contains(descendant_entity));

    self_iter.chain(children_iter)
}

#[cfg(test)]
mod tests {
    use crate::asset::backend::iter_self_and_descendants_with_component::iter_self_and_descendants_with_component;
    use bevy::MinimalPlugins;
    use bevy_app::{App, Update};
    use bevy_derive::{Deref, DerefMut};
    use bevy_ecs::prelude::*;
    use bevy_platform::collections::HashSet;

    /// A marker component to search for in our test.
    #[derive(Component)]
    struct MyComponent;

    #[derive(Resource)]
    struct TestRoot(Entity);

    #[derive(Resource, Default, Deref, DerefMut)]
    struct TestResult(Vec<Entity>);

    /// A one-shot system that calls the utility function and stores the result.
    fn test_system(
        root: Res<TestRoot>,
        mut result: ResMut<TestResult>,
        q_children: Query<&Children>,
        q_component: Query<Entity, With<MyComponent>>,
    ) {
        let found_iter =
            iter_self_and_descendants_with_component(root.0, &q_children, &q_component);

        let found_entities = found_iter.collect();

        **result = found_entities;
    }

    #[test]
    fn test_find_entities_with_component_recursive_should_return_correct_entities() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // - root (With<MyComponent>)
        //   - child_a (With<MyComponent>)
        //     - grandchild_a1
        //     - grandchild_a2 (With<MyComponent>)
        //   - child_b
        //     - grandchild_b1 (With<MyComponent>)
        // - unrelated (With<MyComponent>)

        let world = app.world_mut();

        let grandchild_a1 = world.spawn_empty().id();
        let grandchild_a2 = world.spawn(MyComponent).id();
        let grandchild_b1 = world.spawn(MyComponent).id();

        let child_a = world
            .spawn(MyComponent)
            .add_children(&[grandchild_a1, grandchild_a2])
            .id();

        let child_b = world.spawn_empty().add_children(&[grandchild_b1]).id();

        let root = world
            .spawn(MyComponent)
            .add_children(&[child_a, child_b])
            .id();

        let expected_entities: HashSet<Entity> = [root, child_a, grandchild_a2, grandchild_b1]
            .into_iter()
            .collect();

        // This entity has the component but is not in the hierarchy,
        // so it should not be found.
        let _unrelated = world.spawn(MyComponent).id();

        world.insert_resource(TestRoot(root));
        world.init_resource::<TestResult>();
        app.add_systems(Update, test_system);

        // Act
        app.update();
        let world = app.world();
        let found_entities = world.resource::<TestResult>().0.clone();
        let found_entities_set: HashSet<Entity> = found_entities.into_iter().collect();

        // Assert
        assert_eq!(
            found_entities_set, expected_entities,
            "The function did not find the correct set of entities."
        );

        assert_eq!(found_entities_set.len(), 4, "Expected to find 4 entities.");
    }
}
