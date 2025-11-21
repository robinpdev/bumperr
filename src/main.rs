mod common;
mod module;
mod ui;
mod pipeline;

use common::*;

use bevy::{dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig}, prelude::*, sprite_render::Material2dPlugin};

struct OverlayColor;

// use bevy::prelude::ComputedNode; TODO find feature flag for this
impl OverlayColor {
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}


fn main() {
    let mut bevyapp = App::new();

    let mut default_plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "I am the window!".into(),
                name: Some("bevy.app".into()),
                resolution: (1000, 700).into(),
                // present_mode: PresentMode::AutoNoVsync,
                // Tells Wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                // window_theme: Some(WindowTheme::Dark),
                // enabled_buttons: bevy::window::EnabledButtons {
                //     maximize: false,
                //     ..Default::default()
                // },
                // This will spawn an invisible window
                // The window will be made visible in the make_visible() system after 3 frames.
                // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                visible: true,
                ..default()
            }),
            ..default()
        });

    // Conditionally add the AssetPlugin for Linux
    #[cfg(all(target_os = "linux"))]
    {
        print!("ADDING WATCHER PLUGIN");
        default_plugins = default_plugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..Default::default()
        });
    }

    bevyapp
        .insert_resource(ClearColor(Color::srgba(0.2, 0.2, 0.2, 1.0)))
        .add_plugins((
            default_plugins,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 18.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        // font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: OverlayColor::GREEN,
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: false,
                        // The minimum acceptable fps
                        min_fps: 30.0,
                        // The target fps
                        target_fps: 144.0,
                    },
                },
            },
        ))
        .add_plugins(Material2dPlugin::<CustomMaterial>::default())
        // .edit_schedule(Update, |schedule| {
        //     schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        // })
        .init_state::<AppState>()
        .add_systems(Startup, restart)
        .add_systems(OnEnter(AppState::Restarting), restart)
        .add_systems(OnEnter(AppState::Startup), setup)
        .add_systems(OnExit(AppState::Running), teardown)
        .add_systems(PreUpdate, trigger_restart)
        .add_systems(PreStartup, spawn_immortals)
        .add_plugins(module::ModulePlugin)
        .add_systems(Startup, (pipeline::create_render_target,))
        .add_plugins(ui::BumpUiPlugin);

    bevyapp.run();
}

/// Boilerplate for setting up a basic restarting architecture:
/// Moves the state into AppState::Running so that the OnEnter(AppState::Running) system is called
fn restart(mut next_state: ResMut<NextState<AppState>>) {
    println!("restart!");
    next_state.set(AppState::Startup);
}

/// Boilerplate for setting up a basic restarting architecture:
/// Moves the state into AppState::Running so that the OnEnter(AppState::Running) system is called
fn trigger_restart(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit_events: ResMut<Messages<bevy::app::AppExit>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        println!("user triggered restart");
        next_state.set(AppState::Restarting);
    } else if input.just_pressed(KeyCode::KeyT) {
        println!("user triggered FULL restart");
        app_exit_events.write(bevy::app::AppExit::Success);
    }
}

/// Code that is actually! run once on startup of your program
/// You can spawn entities with the Immortal component (above) here and they will not be removed when restarting
fn spawn_immortals(
    mut commands: Commands,
) {
    println!("immortal");

    // main camera
    commands.spawn((Camera2d, Immortal));
}

fn get_component_names(world: &World, entity: Entity) -> Option<Vec<String>> {
    world
        .inspect_entity(entity)
        .ok() // Convert Result<EntityInspector, EntityDoesNotExistError> to Option<EntityInspector>
        .map(|entity_inspector| {
            entity_inspector
                .map(|component_info| component_info.name().to_string()) // Get the name and convert to String
                .collect::<Vec<String>>() // Collect into a Vec<String>
        })
}

/// User-defined teardown code can live here
/// If you kill all the Windows it will quit the app, so we use Without<PrimaryWindow> here
/// We also don't despawn the "immortals"
fn teardown(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            Without<bevy::window::PrimaryWindow>,
            Without<bevy::picking::pointer::PointerInteraction>,
            Without<bevy::ecs::observer::Observer>,
            Without<bevy::window::Monitor>,
            Without<Immortal>,
            // Without<ComputedNode>,
            // Without<EguiContext>,
        ),
    >,
    world: &World
) {
    // if you want to see what components that entities about to be despawned have

    for entity in query.iter() {
        if let Some(component_names) = get_component_names(world, entity) {
            println!("Component names for entity {:?}: {:?}", entity, component_names);
        } else {
            // This branch is now reached if the entity doesn't exist.
            println!("Entity {:?} does not exist.", entity);
        }
    }

    println!("teardown!");
    for entity in query.iter() {
        // Drain to clear the vec
        commands.entity(entity).despawn();
        println!("Despawned entity: {:?}", entity);
    }
}

/// Runs each time the scene is (re)started
/// Sets up a circle that gets rendered to a texture and then shown on the main context

fn setup(mut next_state: ResMut<NextState<AppState>>) {
    println!("setup!");

    next_state.set(AppState::Running);
}
