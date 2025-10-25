use crate::prelude::*;
use bevy::camera::primitives::{Aabb, MeshAabb};
use bevy::prelude::*;
use std::marker::PhantomData;

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
    Option<&'w StrengthMultiplier>,
    Option<&'w MicroStrengthMultiplier>,
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
            strength: strength
                .map(|x| **x * self.strength)
                .unwrap_or(self.strength),
            micro_strength: micro_strength
                .map(|x| **x * self.micro_strength)
                .unwrap_or(self.micro_strength),
            s_curve_strength: s_curve_strength
                .map(|x| **x * self.s_curve_strength)
                .unwrap_or(self.s_curve_strength),
            s_curve_speed: s_curve_speed
                .map(|x| **x * self.s_curve_speed)
                .unwrap_or(self.s_curve_speed),
            s_curve_frequency: s_curve_frequency
                .map(|x| **x)
                .unwrap_or(self.s_curve_frequency),
            bop_strength: bop_strength
                .map(|x| **x * self.bop_strength)
                .unwrap_or(self.bop_strength),
            bop_speed: bop_speed
                .map(|x| **x * self.bop_speed)
                .unwrap_or(self.bop_speed),
            twist_strength: twist_strength
                .map(|x| **x * self.twist_strength)
                .unwrap_or(self.twist_strength),
            bend_exponent: bend_exponent
                .map(|x| **x * self.bend_exponent)
                .unwrap_or(self.bend_exponent),
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

pub fn queue_material_creation_requests<TOut, TIn>(
    mut cmd: Commands,
    q_roots: Query<(Entity, &ScatterRoot), Without<ScatterRootProcessed>>,
    q_layers: Query<
        (&Children, MaterialOptionData, WindData),
        (
            With<ScatterLayer>,
            Without<ScatterLayerProcessed>,
            With<ScatterLayerType<TOut, TIn>>,
        ),
    >,
    q_collect: Query<
        CollectableQueryData<TIn>,
        (
            Without<ScatterLayerChildProcessed>,
            Without<MaterialCreationRequest<TOut, TIn>>,
        ),
    >,
    wind: Res<Wind>,
) where
    TIn: Material,
    TOut: ScatterMaterial<TIn> + Asset + Clone,
{
    for (root, children) in &q_roots {
        debug!(
            "Queueing ScatterAsset creation requests in root {:?}...",
            root
        );

        for layer in children.iter() {
            let mut wind = wind.clone();
            let Ok((scatter_items, material_option_data, wind_data)) = q_layers.get(layer) else {
                continue;
            };

            wind = wind.with(wind_data);
            let options = MaterialOptions::from(material_option_data);

            for item in scatter_items {
                queue_requests_recursive::<TOut, TIn>(
                    layer, *item, &mut cmd, &wind, &options, None, None, &q_collect,
                );
            }
        }
    }
}

fn queue_requests_recursive<TOut, TIn>(
    layer: Entity,
    entity: Entity,
    cmd: &mut Commands,
    wind: &Wind,
    options: &MaterialOptions,
    current_name: Option<Name>,
    current_lod_level: Option<LevelOfDetail>,
    q_children: &Query<
        CollectableQueryData<TIn>,
        (
            Without<ScatterLayerChildProcessed>,
            Without<MaterialCreationRequest<TOut, TIn>>,
        ),
    >,
) -> bool
where
    TIn: Material,
    TOut: ScatterMaterial<TIn> + Asset + Clone,
{
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
        return false;
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

    let mut has_children_with_materials = false;
    if let Some(children) = children {
        for child in children {
            let found_material = queue_requests_recursive::<TOut, TIn>(
                layer,
                *child,
                cmd,
                &wind,
                &options,
                name.clone(),
                Some(lod),
                q_children,
            );

            if found_material {
                has_children_with_materials = true;
            }
        }
    }

    if has_children_with_materials {
        cmd.entity(entity).insert(ScatterLayerChildProcessed);
    }

    let (Some(material), Some(mesh), Some(aabb)) = (material, mesh, aabb) else {
        return has_children_with_materials;
    };

    cmd.entity(entity)
        .insert((MaterialCreationRequest::<TOut, TIn> {
            source_material_handle: material.0.clone(),
            wind,
            options,
            mesh_handle: mesh.0.clone(),
            aabb: *aabb,
            name,
            lod_level: lod,
            layer,
            _phantom: PhantomData,
        },));

    true
}

// Decouples the "read" phase (collection) with the "write" phase (processing).
#[derive(Component, Clone)]
pub struct MaterialCreationRequest<TOut, TIn>
where
    TIn: Material,
    TOut: ScatterMaterial<TIn> + Asset + Clone,
{
    source_material_handle: Handle<TIn>,
    wind: Wind,
    options: MaterialOptions,
    mesh_handle: Handle<Mesh>,
    aabb: Aabb,
    name: Option<Name>,
    lod_level: LevelOfDetail,
    layer: Entity,
    _phantom: PhantomData<TOut>,
}

pub fn process_distinct_material_requests<TOut, TIn>(
    mut cmd: Commands,
    requests_query: Query<(Entity, &MaterialCreationRequest<TOut, TIn>)>,
    materials_in: Res<Assets<TIn>>,
    mut materials_out: ResMut<Assets<TOut>>,
    wind_noise_texture: Res<WindTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut prototype_assets: ResMut<Assets<ScatterAsset<TOut>>>,
) where
    TIn: Material,
    TOut: ScatterMaterial<TIn> + Asset + Clone,
{
    for (entity, request) in &requests_query {
        let source_material = materials_in.get(&request.source_material_handle);

        let new_material = TOut::create_material(
            source_material.cloned(),
            request.wind.clone(),
            wind_noise_texture.0.clone(),
            request.aabb,
            request.options.clone(),
        );
        let material_handle = materials_out.add(new_material);

        let mesh = meshes.get(&request.mesh_handle).cloned().unwrap();
        let mesh_handle = meshes.add(mesh);
        let mesh_aabb = meshes.get(&mesh_handle).unwrap().compute_aabb().unwrap();

        let asset = ScatterAsset {
            mesh: mesh_handle,
            material: material_handle,
            wind: request.wind.clone(),
            aabb: mesh_aabb,
            name: request.name.clone(),
            lod_level: request.lod_level,
            material_options: request.options.clone(),
            layer: request.layer,
        };

        let asset_handle = prototype_assets.add(asset);

        cmd.entity(entity)
            .remove::<MaterialCreationRequest<TOut, TIn>>()
            .remove::<MeshMaterial3d<TIn>>()
            .insert((
                // TODO only insert if actually wind affected
                WindAffectedRegistered(asset_handle.clone()),
                WindAffected,
                ScatterItem,
                ScatterItemAsset::<TOut>(asset_handle.clone()),
                request.lod_level,
                ChildOf(request.layer),
                ScatterItemOf(request.layer),
                // TODO should remove or use in editor after registration is complete.
                Visibility::Hidden,
                ScatterLayerChildProcessed,
            ));
    }
}

pub fn process_same_type_material_requests<T>(
    mut cmd: Commands,
    requests_query: Query<(Entity, &MaterialCreationRequest<T, T>)>,
    mut materials: ResMut<Assets<T>>,
    wind_noise_texture: Res<WindTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut prototype_assets: ResMut<Assets<ScatterAsset<T>>>,
) where
    T: ScatterMaterial<T> + Material + Clone,
{
    let requests: Vec<(Entity, MaterialCreationRequest<T, T>)> =
        requests_query.iter().map(|(e, r)| (e, r.clone())).collect();

    for (entity, request) in requests {
        let source_material = materials.get(&request.source_material_handle).cloned();

        let new_material = T::create_material(
            source_material,
            request.wind.clone(),
            wind_noise_texture.0.clone(),
            request.aabb,
            request.options.clone(),
        );
        let material_handle = materials.add(new_material);

        let mesh = meshes.get(&request.mesh_handle).cloned().unwrap();
        let mesh_handle = meshes.add(mesh);
        let mesh_aabb = meshes.get(&mesh_handle).unwrap().compute_aabb().unwrap();

        let asset = ScatterAsset {
            mesh: mesh_handle,
            material: material_handle,
            wind: request.wind.clone(),
            aabb: mesh_aabb,
            name: request.name.clone(),
            lod_level: request.lod_level,
            material_options: request.options.clone(),
            layer: request.layer,
        };

        let asset_handle = prototype_assets.add(asset);

        cmd.entity(entity)
            .remove::<MaterialCreationRequest<T, T>>()
            .insert((
                // TODO only insert if actually wind affected
                WindAffectedRegistered(asset_handle.clone()),
                WindAffected,
                ScatterItem,
                ScatterItemAsset::<T>(asset_handle.clone()),
                request.lod_level,
                ChildOf(request.layer),
                ScatterItemOf(request.layer),
                // TODO should remove or use in editor after registration is complete.
                Visibility::Hidden,
                ScatterLayerChildProcessed,
            ));
    }
}
