use bevy::asset::uuid::Uuid;
use bevy::camera::RenderTarget;
use bevy::prelude::*;

use crate::module::*;
use crate::rendering::*;

use bevy::sprite_render::Material2dPlugin;
use bevy::{reflect::TypePath, render::render_resource::AsBindGroup};

use bevy::{shader::ShaderRef, sprite_render::Material2d};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

pub struct NoiseModule;

impl Plugin for NoiseModule {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<NoiseMaterial>::default(),
            ShaderChainPlugin,
        ))
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
    mut images: ResMut<Assets<Image>>,
) {
    let image = Image::new_target_texture(512, 512, TextureFormat::bevy_default(), None);
    let image_handle = images.add(image);

    let drawlayer = RenderLayers::layer(1);

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
            drawlayer.clone(),
        ))
        .id();

    commands.spawn((
        Camera2d::default(),
        RenderTarget::Image(image_handle.clone().into()),
        Camera {
            clear_color: Color::hsla(0.0, 0.0, 0.0, 0.0).into(),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
        ShaderChainCamera {
            shaders: vec![
                "shaders/post_processing_2.wgsl".to_string(),
                "shaders/post_processing.wgsl".to_string(),
                ],
            iid: 1,
        },
        drawlayer,
        
    ));

    //Sprite to display the rendered texture
    let sprite = commands.spawn(
        (
            ModulePart(spawn.root_id),
            Sprite::from_image(image_handle.clone()),
        )
    ).id();

    commands.entity(spawn.root_id).add_child(sprite);
}

fn resize_surface(
    resize: On<ResizeModuleInternal>,
    mut materials: ResMut<Assets<NoiseMaterial>>,
    mut surfaces: Query<(&mut Transform, &MeshMaterial2d<NoiseMaterial>), With<Mesh2d>>,
    roots: Query<&ModuleWithParts, With<ModuleWin>>,
    sprites: Query<&Sprite>,
    mut images: ResMut<Assets<Image>>
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
            else if let Ok(sprite) = sprites.get(child) {
                if let Some(image) = images.get_mut(sprite.image.id()){
                    image.resize(Extent3d { width: resize.width as u32, height:resize.height as u32, depth_or_array_layers: 1 });
                }
            }
        }
    }
}
