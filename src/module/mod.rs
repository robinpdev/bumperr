use crate::common::*;
use crate::module::noise::spawn_noise_module;
use crate::rendering::ShaderChainPlugin;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
// use bevy_simple_subsecond_system::prelude::*;

//import noisemodule
mod noise;
mod pong;

use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum ModuleClass {
    Pong,
    Noise,
}

trait ModuleClassAttrs {
    const SPAWN_OBSERVER: Observer;
}

trait HasModuleClass {
    fn get_module_class(&self) -> ModuleClass;
}

#[derive(Event)]
pub struct SpawnModuleEvent {
    pub moduleclass: ModuleClass,
}

#[derive(EntityEvent)]
pub struct SpawnModuleInternalEvent {
    #[event_target]
    pub spawner: Entity,
    pub moduleclass: ModuleClass,
    pub layer: RenderLayers,
    pub root_id: Entity,
}

impl HasModuleClass for SpawnModuleInternalEvent {
    fn get_module_class(&self) -> ModuleClass {
        self.moduleclass
    }
}

#[derive(EntityEvent)]
pub struct ResizeModule {
    pub entity: Entity,
    pub width: f32,
    pub height: f32,
}

#[derive(EntityEvent)]
pub struct ResizeModuleInternal {
    #[event_target]
    pub spawner: Entity,
    pub moduleroot: Entity,
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
pub struct ModuleLayerCounter(pub u8);

#[derive(Resource)]
pub struct ModuleSpawnerConfig {
    pub observers: HashMap<ModuleClass, Vec<Entity>>,
}


#[derive(Component)]
#[relationship_target(relationship = ModulePart, linked_spawn)]
pub struct ModuleWithParts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = ModuleWithParts)]
pub struct ModulePart(pub Entity);


pub struct ModulePlugin;

impl Plugin for ModulePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ModuleLayerCounter(1))
            .insert_resource(ModuleSpawnerConfig{observers: HashMap::new()})
            .add_observer(spawn_module_observer)
            .add_plugins(noise::NoiseModule)
            .add_plugins(pong::PongModule)
            // .add_systems(Update, (
            //     handle_click
            //         .run_if(input_just_pressed(MouseButton::Left)),
            // ))
            ;
    }
}

#[derive(Component)]
pub struct FirstPassEntity {
    module_id: Entity,
}

fn trigger_spawner<'a, E: Event<Trigger<'a>: Default>, F>(
    mut commands: Commands,
    spawnconfig: &ModuleSpawnerConfig,
    class: ModuleClass,
    make_event: F,
) where
    F: Fn(Entity) -> E,
{
    if let Some(spawners) = spawnconfig.observers.get(&class) {
        for spawner in spawners.iter() {
            let ev = make_event(*spawner);
            commands.trigger(ev);
        }
    }
}

pub fn spawn_module_observer(
    spawn: On<SpawnModuleEvent>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layer_counter: ResMut<ModuleLayerCounter>,
    spawnconfig: Res<ModuleSpawnerConfig>,
) {
    println!("module setup!");
    // rendered texture
    let size = Extent3d {
        width: BOXWIDTH as u32,
        height: BOXHEIGHT as u32,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );

    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    //Sprite to display the rendered texture
    let mut sprite = Sprite::from_image(image_handle.clone());
    sprite.custom_size = Some(Vec2 {
        x: BOXWIDTH,
        y: BOXHEIGHT,
    });
    let spriteid = commands
        .spawn((
            // sprite,
            ModuleWin {
                class: spawn.moduleclass,
                width: BOXWIDTH,
                height: BOXHEIGHT,
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, layer_counter.0 as f32 * 0.01)),
        ))
        .observe(resize_image_observer)
        .id();

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = RenderLayers::layer(layer_counter.0 as usize);
    layer_counter.0 += 1;

    //first pass camera
    // commands.spawn((
    //     Camera2d::default(),
    //     Camera {
    //         target: image_handle.clone().into(),
    //         clear_color: Color::WHITE.into(),
    //         ..default()
    //     },
    //     Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
    //     first_pass_layer.clone(),
    // ));

    trigger_spawner::<SpawnModuleInternalEvent, _>(
        commands,
        &spawnconfig,
        spawn.moduleclass,
        |spawner| SpawnModuleInternalEvent {
            spawner,
            moduleclass: spawn.moduleclass,
            layer: first_pass_layer.clone(),
            root_id: spriteid,
        },
    );
}

fn resize_image_observer(
    resize: On<ResizeModule>,
    commands: Commands,
    mut assets: ResMut<Assets<Image>>,
    wins: Query<(&mut ModuleWin)>,
    spawnconfig: Res<ModuleSpawnerConfig>,
) {
    if let Ok((win)) = wins.get(resize.entity) {

        trigger_spawner::<ResizeModuleInternal, _>(commands, &spawnconfig, win.class, |spawner| {
            ResizeModuleInternal {
                spawner,
                moduleroot: resize.entity,
                width: resize.width,
                height: resize.height,
            }
        });
    }
}
