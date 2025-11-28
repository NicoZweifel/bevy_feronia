use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use bevy_render::extract_component::ExtractComponent;
use derive_more::From;

/// Marker component identifying the entity representing the center of the chunking and lod systems.
///
/// This should be added to the camera or the player controller.
#[derive(Component, Reflect, Clone, ExtractComponent)]
#[reflect(Component)]
pub struct Center;

/// Component specifying the LOD for a [`ScatterItem`].
#[derive(
    Component,
    Deref,
    DerefMut,
    Clone,
    Copy,
    Debug,
    Default,
    Reflect,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    From,
)]
#[reflect(Component, Clone, Debug, PartialEq, Hash)]
pub struct LevelOfDetail(pub u32);

/// Marker component for debug visualization.
///
/// Makes shaders return `debug_color` in the fragment shader.
///
/// Enables `#ifdef MATERIAL_DEBUG` in shaders.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct EnableDebug;

/// Marker component to make instances always face the camera.
///
/// Enables `#ifdef BILLBOARDING` in shaders.
///
/// Not supported in combination with [`EdgeCorrectionFactor`].
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct EnableBillboarding;

/// Marker component to force simple, undisplaced world-space normals.
///
/// Will have incorrect lighting on displaced vertices,
/// as the normals will not match the displaced vertex positions.
///
/// **Note:** If neither [`FastNormals`] nor [`AnalyticalNormals`] is present,
/// the shader defaults to the numerical path, which is the most accurate, but most expensive path,
/// as it runs the full displacement logic on the neighbors to find the surface direction,
/// which should only be used for complex foliage like non-billboarded bushes, trees.
///
/// **Note:** For correct fallback behavior (if the mesh lacks tangents or normals),
/// the mesh should ideally be modeled with its "growth" axis along Y-Up (`+Y`)
/// and its "face" pointing along Z-Up (`+Z`).
///
/// Typically used for performance reasons and/or on static or barely wind-affected objects.
///
/// Enables `#ifdef FAST_NORMALS` in shaders.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct FastNormals;

/// Marker component to enable approximated, mathematically derived normals.
///
/// Should be faster than numerical sampling but less accurate,
/// as it only accounts for `static_bend`, `twist`,
/// and `macro_wind`, ignoring high-frequency displacements.
///
/// **Note:** If neither [`FastNormals`] nor [`AnalyticalNormals`] is present,
/// the shader defaults to the numerical path, which is the most accurate, but most expensive path,
/// as it runs the full displacement logic on the neighbors to find the surface direction,
/// which should only be used for complex foliage like non-billboarded bushes, trees.
///
/// **Note:** For correct fallback behavior (if the mesh lacks tangents or normals),
/// the mesh should ideally be modeled with its "growth" axis along Y-Up (`+Y`)
/// and its "face" pointing along Z-Up (`+Z`).
///
/// Typically used for billboarded foliage or flat meshes like grass.
///
/// Enables `#ifdef ANALYTICAL_NORMALS` in shaders.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct AnalyticalNormals;
