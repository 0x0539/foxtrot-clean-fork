#![allow(clippy::extra_unused_type_parameters)]
use crate::file_system_interaction::asset_loading::TextureAssets;
use crate::GameState;
use bevy::asset::HandleId;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::utils::HashMap;
use regex::Regex;

/// Handles instantiation of shaders. The shaders can be found in the [`shaders`](https://github.com/janhohenheim/foxtrot/tree/main/assets/shaders) directory.
/// Shaders are stored in [`Material`]s which can be used on objects by attaching a `Handle<Material>` to an entity.
/// The handles can be stored and retrieved in the [`Materials`] resource.
pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<GlowyMaterial>::default())
            .add_plugin(MaterialPlugin::<RepeatedMaterial>::default())
            .add_system_set(SystemSet::on_exit(GameState::Loading).with_system(setup_shader))
            .add_system_set(
                SystemSet::on_update(GameState::Playing).with_system(set_texture_to_repeat),
            );
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Materials {
    pub glowy: Handle<GlowyMaterial>,
    /// (Texture asset ID, Repeats) -> RepeatedMaterial
    pub repeated: HashMap<(HandleId, Repeats), Handle<RepeatedMaterial>>,
}

fn setup_shader(
    mut commands: Commands,
    mut glow_materials: ResMut<Assets<GlowyMaterial>>,
    texture_assets: Res<TextureAssets>,
) {
    let glowy_material = glow_materials.add(GlowyMaterial {
        env_texture: texture_assets.glowy_interior.clone(),
    });

    commands.insert_resource(Materials {
        glowy: glowy_material,
        repeated: HashMap::new(),
    });
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "bd5c76fd-6fdd-4de4-9744-4e8beea8daaf"]
/// Material for [`glowy.wgsl`](https://github.com/janhohenheim/foxtrot/blob/main/assets/shaders/glowy.wgsl).
pub struct GlowyMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub env_texture: Handle<Image>,
}

impl Material for GlowyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/glowy.wgsl".into()
    }
}

#[repr(C, align(16))] // All WebGPU uniforms must be aligned to 16 bytes
#[derive(Clone, Copy, ShaderType, Debug, Hash, Eq, PartialEq, Default)]
pub struct Repeats {
    pub horizontal: u32,
    pub vertical: u32,
    pub _wasm_padding1: u32,
    pub _wasm_padding2: u32,
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "82d336c5-fd6c-41a3-bdd4-267cd4c9be22"]
/// Material for [`repeated.wgsl`](https://github.com/janhohenheim/foxtrot/blob/main/assets/shaders/repeated.wgsl).
pub struct RepeatedMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub repeats: Repeats,
}

impl Material for RepeatedMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/repeated.wgsl".into()
    }
}

pub fn set_texture_to_repeat(
    mut commands: Commands,
    added_name: Query<(&Name, &Children), Added<Name>>,
    material_handles: Query<&Handle<StandardMaterial>>,
    mut materials: ResMut<Materials>,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut repeated_materials: ResMut<Assets<RepeatedMaterial>>,
) {
    let re = Regex::new(r"\[repeat:(\d+),(\d+)\]").unwrap();
    for (name, children) in &added_name {
        if let Some(captures) = re.captures(&name.to_lowercase()) {
            let repeats = Repeats {
                horizontal: captures[1].parse().unwrap(),
                vertical: captures[2].parse().unwrap(),
                ..default()
            };
            for child in children.iter() {
                if let Ok(standard_material_handle) = material_handles.get(*child) {
                    let standard_material =
                        standard_materials.get(standard_material_handle).unwrap();
                    let texture = standard_material.base_color_texture.as_ref().unwrap();
                    let key = (texture.id(), repeats);

                    let repeated_material = materials.repeated.entry(key).or_insert_with(|| {
                        repeated_materials.add(RepeatedMaterial {
                            texture: texture.clone(),
                            repeats,
                        })
                    });

                    commands
                        .entity(*child)
                        .remove::<Handle<StandardMaterial>>()
                        .insert(repeated_material.clone());
                }
            }
        }
    }
}
