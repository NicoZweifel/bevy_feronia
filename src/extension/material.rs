use crate::prelude::*;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;

pub type ExtendedWindAffectedMaterial = ExtendedMaterial<StandardMaterial, WindAffectedExtension>;
