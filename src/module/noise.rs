use bevy::prelude::*;

use crate::module::*;

use bevy::sprite_render::Material2dPlugin;
use bevy::{reflect::TypePath, render::render_resource::AsBindGroup};

use bevy::{shader::ShaderRef, sprite_render::Material2d};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

pub struct NoiseModule;

impl Plugin for NoiseModule {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<NoiseMaterial>::default())
            .add_systems(OnEnter(AppState::Startup), setup)
            .add_systems(EguiPrimaryContextPass, ui_noise);
    }
}

fn ui_noise(
    mut contexts: EguiContexts,
    windows: Query<&mut Window>,
    mut materials: ResMut<Assets<NoiseMaterial>>,
) -> Result {
    for (shaderid, shader) in materials.iter_mut() {
        if let Ok(_win) = windows.single() {
            egui::Window::new(format!("Noise params"))
            .id(egui::Id::new(shaderid))
            .show(contexts.ctx_mut()?, |ui| {
                ui.add(egui::Slider::new(&mut shader.speed, 0.0..=10.0).suffix("°"));
            });
        }
    }
    Ok(())
}

fn setup(mut commands: Commands, mut spawnerconfig: ResMut<ModuleSpawnerConfig>) {
    let eid = commands
        .spawn((ModuleSpawner {
            class: ModuleClass::Noise,
        },))
        .observe(spawn_noise_module)
        .observe(resize_surface)
        .id();

    spawnerconfig
        .observers
        .insert(ModuleClass::Noise, vec![eid]);
}

// fn resize_rect(

// )

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct NoiseMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(1)]
    pub width: f32,
    #[uniform(2)]
    pub height: f32,
    #[uniform(3)]
    pub speed: f32,
}

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/noise.wgsl";

/// The Material2d trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material2d api docs for details!
impl Material2d for NoiseMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

pub fn spawn_noise_module(
    spawn: On<SpawnModuleInternalEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shadermaterials: ResMut<Assets<NoiseMaterial>>,
) {
    if spawn.moduleclass != ModuleClass::Noise {
        return;
    };
    // Spawn the noise module entities here
    println!("Spawning Noise Module");

    let shader = shadermaterials.add(NoiseMaterial {
        color: LinearRgba::GREEN,
        width: BOXWIDTH,
        height: BOXHEIGHT,
        speed: 1.0,
    });

    let shadersurface: Entity = commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(1., 1.))),
            //MeshMaterial2d(colormaterials.add(Color::srgb(0.0, 1.0, 0.0))),
            MeshMaterial2d(shader.clone()),
            Transform::default().with_scale(Vec3::new(BOXWIDTH, BOXHEIGHT, 1.0)),
            FirstPassEntity {
                module_id: spawn.root_id,
            },
            ModulePart(spawn.root_id),
            // spawn.layer.clone()
        ))
        .id();

    commands.entity(spawn.root_id).add_child(shadersurface);
}

fn resize_surface(
    resize: On<ResizeModuleInternal>,
    mut materials: ResMut<Assets<NoiseMaterial>>,
    mut surfaces: Query<(&mut Transform, &MeshMaterial2d<NoiseMaterial>), With<Mesh2d>>,
    roots: Query<&ModuleWithParts, With<ModuleWin>>,
) {
    if let Ok(rootchildren) = roots.get(resize.moduleroot) {
        for child in rootchildren.iter() {
            if let Ok((mut transform, materialref)) = surfaces.get_mut(child) {
                let newscale = Vec3::new(resize.width, resize.height, 1.0);
                transform.scale = newscale;
                if let Some(shader) = materials.get_mut(materialref.id()) {
                    shader.width = resize.width;
                    shader.height = resize.height;
                }
            }
        }
    }
}
