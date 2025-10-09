use crate::prelude::*;
use bevy::camera::primitives::{Aabb, MeshAabb};
use bevy::prelude::*;

pub type CollectableQueryData<'w, T> = (
    Entity,
    Option<&'w MeshMaterial3d<T>>,
    Option<&'w Mesh3d>,
    Option<&'w Aabb>,
    Option<&'w Children>,
    Option<&'w WindConfig>,
    Option<&'w Name>,
    Option<&'w LevelOfDetail>,
    Option<&'w WindAffected>,
    MaterialOptionData<'w>,
    WindData<'w>,
);

pub type MaterialOptionData<'w> = (
    Option<&'w EnableDebug>,
    Option<&'w EnableBillboarding>,
    Option<&'w EdgeCorrectionFactor>,
    Option<&'w CurveFactor>,
);

pub type WindData<'w> = (
    Option<&'w Strength>,
    Option<&'w MicroStrength>,
    Option<&'w SCurveStrength>,
    Option<&'w SCurveSpeed>,
    Option<&'w SCurveFrequency>,
    Option<&'w BopStrength>,
    Option<&'w BopSpeed>,
    Option<&'w TwistStrength>,
    Option<&'w BendExponent>,
    Option<&'w LowQuality>,
);

impl Wind {
    pub fn with(
        &self,
        (
            strength,
            micro_strength,
            s_curve_strength,
            s_curve_speed,
            s_curve_frequency,
            bop_strength,
            bop_speed,
            twist_strength,
            bend_exponent,
            low_quality,
        ): WindData,
    ) -> Self {
        Wind {
            strength: strength.map(|x| **x * self.strength).unwrap_or(self.strength),
            micro_strength: micro_strength.map(|x| **x * self.micro_strength).unwrap_or(self.micro_strength),
            s_curve_strength: s_curve_strength
                .map(|x| **x * self.s_curve_strength)
                .unwrap_or(self.s_curve_strength),
            s_curve_speed: s_curve_speed.map(|x| **x * self.s_curve_speed).unwrap_or(self.s_curve_speed),
            s_curve_frequency: s_curve_frequency
                .map(|x| **x)
                .unwrap_or(self.s_curve_frequency),
            bop_strength: bop_strength.map(|x| **x * self.bop_strength).unwrap_or(self.bop_strength),
            bop_speed: bop_speed.map(|x| **x * self.bop_speed).unwrap_or(self.bop_speed),
            twist_strength: twist_strength.map(|x| **x * self.twist_strength).unwrap_or(self.twist_strength),
            bend_exponent: bend_exponent.map(|x| **x * self.bend_exponent).unwrap_or(self.bend_exponent),
            low_quality: low_quality.map(|_| true).unwrap_or(self.low_quality),
            ..*self
        }
    }
}

impl From<MaterialOptionData<'_>> for MaterialOptions {
    fn from(
        (enable_debug, enable_billboarding, edge_correction_factor, curve_factor):MaterialOptionData,
    ) -> Self {
        Self {
            debug: enable_debug.is_some(),
            enable_billboarding: enable_billboarding.is_some(),
            edge_correction_factor: edge_correction_factor.map(|x| **x).unwrap_or(0.),
            curve_factor: curve_factor.map(|x| **x).unwrap_or(0.),
            ..default()
        }
    }
}

impl MaterialOptions {
    pub fn with(
        &self,
        (enable_debug, enable_billboarding, edge_correction_factor, curve_factor):MaterialOptionData,
    ) -> Self {
        Self {
            debug: enable_debug.map(|_| true).unwrap_or(self.debug),
            enable_billboarding: enable_billboarding
                .map(|_| true)
                .unwrap_or(self.enable_billboarding),
            edge_correction_factor: edge_correction_factor
                .map(|x| **x)
                .unwrap_or(self.edge_correction_factor),
            curve_factor: curve_factor.map(|x| **x).unwrap_or(self.curve_factor),
            ..*self
        }
    }
    pub fn with_debug_color(mut self, debug_color: Color) -> Self {
        self.debug_color = debug_color;
        self
    }

    pub fn with_controlled(mut self, controlled: bool) -> Self {
        self.controlled = controlled;
        self
    }
}

pub fn collect_assets<TIn, TOut>(
    mut cmd: Commands,
    q_roots: Query<(Entity, &ScatterRoot), Without<ScatterRootProcessed>>,
    q_layers: Query<
        (&Children, MaterialOptionData, WindData),
        (
            With<ScatterLayer>,
            Without<ScatterLayerProcessed>,
            With<ScatterLayerType<TIn, TOut>>,
        ),
    >,
    q_collect: Query<CollectableQueryData<TIn>, Without<ScatterLayerChildProcessed>>,
    mut materials: ResMut<Assets<TIn>>,
    mut extended_materials: ResMut<Assets<TOut>>,
    wind_noise_texture: Res<WindTexture>,
    wind: Res<Wind>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut prototype_assets: ResMut<Assets<ScatterAsset<TOut>>>,
) where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    for (root, children) in &q_roots {
        debug!("Collecting ScatterAssets in root {:?}...", root);

        for layer in children.iter() {
            let mut wind = wind.clone();
            let Ok((scatter_items, material_option_data, wind_data)) = q_layers.get(layer) else {
                continue;
            };

            wind = wind.with(wind_data);

            let options = MaterialOptions::from(material_option_data);

            let result = scatter_items
                .iter()
                .flat_map(|x| {
                    collect_assets_recursive::<TIn, TOut>(
                        layer,
                        x,
                        &mut cmd,
                        &mut materials,
                        &mut extended_materials,
                        &wind_noise_texture,
                        &wind,
                        &options,
                        None,
                        None,
                        &mut prototype_assets,
                        &mut meshes,
                        &q_collect,
                    )
                })
                .collect::<Vec<_>>();

            if result.is_empty() {
                continue;
            };

            debug!("Found {} assets in layer {:?}", result.len(), layer);
        }
    }
}

fn collect_assets_recursive<TIn, TOut>(
    layer: Entity,
    entity: Entity,
    cmd: &mut Commands,
    materials: &mut Assets<TIn>,
    extended_materials: &mut Assets<TOut>,
    wind_noise_texture: &WindTexture,
    wind: &Wind,
    options: &MaterialOptions,
    current_name: Option<Name>,
    current_lod_level: Option<LevelOfDetail>,
    scatter_assets: &mut Assets<ScatterAsset<TOut>>,
    meshes: &mut Assets<Mesh>,
    q_children: &Query<CollectableQueryData<TIn>, Without<ScatterLayerChildProcessed>>,
) -> Vec<Handle<ScatterAsset<TOut>>>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    let mut types: Vec<Handle<ScatterAsset<TOut>>> = Vec::new();

    // TODO only add displacement/wind affected materials if wind affected
    let Ok((
        entity,
        material,
        mesh,
        aabb,
        children,
        wind_component,
        name,
        lod,
        // TODO
        _wind_affected,
        material_option_data,
        wind_data,
    )) = q_children.get(entity)
    else {
        return types;
    };

    let (mut wind, controlled) = wind_component
        .and_then(|x| x.wind_override.clone().map(|x| (x.clone(), true)))
        .unwrap_or_else(|| ((*wind).clone(), false));

    wind = wind.with(wind_data);

    let lod = lod.map_or(current_lod_level.unwrap_or_default(), |x| *x);

    let name = current_name.map_or(name.cloned(), Some);

    let hue = (entity.index() * 30) as f32 % 360.0;
    let debug_color = Color::hsl(hue, 1.0, 0.5);

    let options = options
        .with(material_option_data)
        .with_debug_color(debug_color)
        .with_controlled(controlled);

    if let Some(children) = children {
        for child in children.iter() {
            types.append(&mut collect_assets_recursive::<TIn, TOut>(
                layer,
                child,
                cmd,
                materials,
                extended_materials,
                wind_noise_texture,
                &wind,
                &options,
                name.clone(),
                Some(lod),
                scatter_assets,
                meshes,
                q_children,
            ));
        }
    }

    if !types.is_empty() {
        cmd.entity(entity).insert(ScatterLayerChildProcessed);
    }

    let Some(material) = material else {
        return types;
    };

    let Some(mesh) = mesh else {
        return types;
    };

    let Some(aabb) = aabb else { return types };

    let new_material = TOut::create_material(
        Some(materials.get(material).unwrap().clone()),
        wind.clone(),
        wind_noise_texture.0.clone(),
        *aabb,
        options.clone(),
    );

    let material = extended_materials.add(new_material);

    let mesh = meshes.get(mesh).cloned().unwrap();
    let mesh = meshes.add(mesh.clone());

    let mesh_aabb = meshes.get(&mesh).unwrap().compute_aabb().unwrap();

    let asset = ScatterAsset {
        mesh,
        material,
        wind,
        aabb: mesh_aabb,
        name,
        lod_level: lod,
        material_options: options,
        layer,
    };

    debug!(
        "Adding asset {:?} lod_level {:?}",
        asset.name, asset.lod_level
    );

    let asset_handle = scatter_assets.add(asset);

    cmd.entity(entity).remove::<MeshMaterial3d<TIn>>().insert((
        // TODO only do this and ignore scatter item logic (some assets might not ever be scattered and just need to be affected by wind).
        WindAffectedRegistered(asset_handle.clone()),
        WindAffected,
        ScatterItem,
        ScatterItemAsset::<TOut>(asset_handle.clone()),
        lod,
        ChildOf(layer),
        ScatterItemOf(layer),
        ScatterLayerChildProcessed,
    ));

    types.push(asset_handle);

    types
}
