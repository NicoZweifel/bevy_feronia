use crate::prelude::*;
use bevy_pbr::{ExtendedMaterial, StandardMaterial};

pub type ExtendedWindAffectedMaterial = ExtendedMaterial<StandardMaterial, WindAffectedExtension>;
