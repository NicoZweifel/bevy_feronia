use crate::core::SpawnTrigger;
use crate::prelude::*;
use bevy_asset::Assets;
use bevy_ecs::prelude::*;
use bevy_pbr::StandardMaterial;
use bevy_platform::collections::HashMap;
use std::fmt::Debug;
#[cfg(feature = "trace")]
use tracing::warn;

/// Event used to trigger the spawning of a batch of `[ScatterAssets]`.
#[derive(Event, Message, Debug, Clone)]
pub struct SpawnScatterAssets<T = StandardMaterial>
where
    T: ScatterMaterialAsset,
{
    /// A list of asset definitions to be scattered.
    pub items: Vec<ScatterItemAsset<T>>,
    /// Contains the computed scatter results (`data`) and contextual information.
    pub trigger: SpawnTrigger,
}

impl<T> SpawnScatterAssets<T>
where
    T: ScatterMaterialAsset,
{
    pub fn new(items: Vec<ScatterItemAsset<T>>, trigger: SpawnTrigger) -> Self {
        Self { items, trigger }
    }

    pub fn with_items(mut self, items: impl Iterator<Item = ScatterItemAsset<T>> + Clone) -> Self {
        self.items = items.collect();
        self
    }

    pub fn create_name_map<'w>(
        &self,
        prototype_assets: &'w Assets<ScatterAsset<T>>,
    ) -> HashMap<Name, Vec<ScatterHandleAsset<'w, T>>> {
        self.items
            .iter()
            .filter_map(|scatter_item_asset| {
                prototype_assets
                    .get(&**scatter_item_asset)
                    .map(|asset| ScatterHandleAsset {
                        handle: (**scatter_item_asset).clone(),
                        asset,
                    })
            })
            .fold(
                HashMap::new(),
                |mut map, ScatterHandleAsset { handle, asset }| {
                    let name = asset.properties.name.as_ref().map_or_else(
                        || {
                            #[cfg(feature = "trace")]
                            warn!("ScatterAsset {:?} has no name!", handle);
                            Name::new("")
                        },
                        |name| clean_lod_suffix(name).into(),
                    );

                    map.entry(name)
                        .or_default()
                        .push(ScatterHandleAsset { handle, asset });

                    map
                },
            )
    }
}

impl<T> From<SpawnTrigger> for SpawnScatterAssets<T>
where
    T: ScatterMaterialAsset,
{
    fn from(value: SpawnTrigger) -> Self {
        Self::new(Vec::new(), value)
    }
}

impl<T> From<On<'_, '_, ScatterResults<T>>> for SpawnScatterAssets<T>
where
    T: ScatterMaterial,
{
    fn from(value: On<'_, '_, ScatterResults<T>>) -> Self {
        Self::from(SpawnTrigger::from(value))
    }
}

/// Removes a case-insensitive LOD suffix (e.g., "_LOD_1", "lod2") from a string slice.
///
/// Works from the end of the string and avoids new allocations.
fn clean_lod_suffix(name: &str) -> &str {
    let Some(last_non_digit_idx) = name.rfind(|c: char| !c.is_ascii_digit()) else {
        return name;
    };

    let (mut base, digits) = name.split_at(last_non_digit_idx + 1);
    if digits.is_empty() {
        return name;
    }

    base = base.trim_end_matches(|c: char| c == '_' || c.is_whitespace());

    if base.len() >= 3 && base[base.len() - 3..].eq_ignore_ascii_case("lod") {
        let final_base = &base[..base.len() - 3];

        return final_base.trim_end_matches(|c: char| c == '_' || c.is_whitespace());
    }

    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_lod_suffixes_should_remove() {
        assert_eq!(clean_lod_suffix("Tree_LOD_1"), "Tree");
        assert_eq!(clean_lod_suffix("Bush_LOD_0"), "Bush");
        assert_eq!(clean_lod_suffix("Rock_LOD_3"), "Rock");
    }

    #[test]
    fn test_case_insensitive_lod_should_remove() {
        assert_eq!(clean_lod_suffix("Tree_lod_1"), "Tree");
        assert_eq!(clean_lod_suffix("Tree_Lod_1"), "Tree");
        assert_eq!(clean_lod_suffix("Tree_lOd_1"), "Tree");
    }

    #[test]
    fn test_separator_variations_should_get_handled() {
        assert_eq!(clean_lod_suffix("TreeLOD1"), "Tree", "No separators");
        assert_eq!(
            clean_lod_suffix("Tree_LOD1"),
            "Tree",
            "Separator before LOD"
        );
        assert_eq!(
            clean_lod_suffix("TreeLOD_1"),
            "Tree",
            "Separator before digit"
        );
        assert_eq!(
            clean_lod_suffix("Tree lod 2"),
            "Tree",
            "Whitespace separators"
        );
        assert_eq!(clean_lod_suffix("Tree_lod 2"), "Tree", "Mixed separators");
    }

    #[test]
    fn test_multiple_digit_lods_should_remove() {
        assert_eq!(clean_lod_suffix("BigRock_LOD_12"), "BigRock");
        assert_eq!(clean_lod_suffix("Shrub_lod_03"), "Shrub");
        assert_eq!(clean_lod_suffix("FernLOD123"), "Fern");
    }

    #[test]
    fn test_trailing_separators_on_base_should_trim() {
        assert_eq!(clean_lod_suffix("MyTree__LOD_1"), "MyTree");
        assert_eq!(clean_lod_suffix("MyTree_ lod 1"), "MyTree");
    }

    #[test]
    fn test_names_without_lod_should_not_change() {
        // "lod" not at the end
        assert_eq!(clean_lod_suffix("MyLodge_1"), "MyLodge_1");
        assert_eq!(clean_lod_suffix("LOD_1_Tree"), "LOD_1_Tree");
        assert_eq!(clean_lod_suffix("Tree_LOD_1_Variant"), "Tree_LOD_1_Variant");

        // No digits
        assert_eq!(clean_lod_suffix("Tree_LOD"), "Tree_LOD");
        assert_eq!(clean_lod_suffix("MyTreeLOD"), "MyTreeLOD");

        // No "lod" keyword
        assert_eq!(clean_lod_suffix("Tree_1"), "Tree_1");
        assert_eq!(clean_lod_suffix("Tree123"), "Tree123");
    }

    #[test]
    fn test_missing_names_should_become_empty() {
        // Base name is effectively an empty string
        assert_eq!(clean_lod_suffix("lod1"), "");
        assert_eq!(clean_lod_suffix("_LOD_2"), "");
        assert_eq!(clean_lod_suffix("LOD_01"), "");
        assert_eq!(clean_lod_suffix("_lod1"), "");
    }
}
