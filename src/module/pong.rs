use bevy::prelude::*;

use crate::module::*;

pub struct PongModule;

impl Plugin for PongModule {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Startup), setup)
            .add_systems(Update, pong_system.run_if(in_state(AppState::Running)));
    }
}

fn setup(
    mut commands: Commands,
    mut spawnerconfig: ResMut<ModuleSpawnerConfig>
){
    let eid = commands.spawn((ModuleSpawner{
            class: ModuleClass::Pong
        },))
        .observe(spawn_module)
        .id();

    spawnerconfig.observers.insert(ModuleClass::Pong, vec![eid]);
}

fn spawn_module(
    spawn: On<SpawnModuleInternalEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shadermaterials: ResMut<Assets<CustomMaterial>>,
) {
    // if spawn.moduleclass != ModuleClass::Pong { return };
    // Spawn the noise module entities here
    println!("Spawning Pong Module");

    //first pass circle mesh
    let ball = commands.spawn((
        Mesh2d(meshes.add(Circle::new(RADIUS))),
        //MeshMaterial2d(colormaterials.add(Color::srgb(0.0, 1.0, 0.0))),
        MeshMaterial2d(shadermaterials.add(CustomMaterial {
            color: LinearRgba::RED,
        })),
        Transform::default(),
        HDirection::Right,
        VDirection::Up,
        FirstPassEntity{module_id: spawn.root_id},
    )).id();

    commands.entity(spawn.root_id).add_child(ball);

}

/// Rotates the inner cube (first pass)

fn pong_system(
    mut query: Query<(&mut Transform, &mut VDirection, &mut HDirection, &FirstPassEntity)>,
    modules: Query<&ModuleWin>,
) {
    // for mut transform in &mut query {
    //     transform.rotate_x(1.5 * time.delta_secs());
    //     transform.rotate_z(0.4 * time.delta_secs());
    // }

    for (mut pos, mut vdir, mut hdir, fpe) in &mut query {
        let mw = modules.get(fpe.module_id).unwrap();
        let boxwidth = mw.width;
        let boxheight = mw.height;

        match *vdir {
            VDirection::Up => pos.translation.y += SPEED,
            VDirection::Down => pos.translation.y -= SPEED,
        }

        match *hdir {
            HDirection::Left => pos.translation.x -= SPEED,
            HDirection::Right => pos.translation.x += SPEED,
        }

        if pos.translation.x + RADIUS > boxwidth / 2 as f32 {
            *hdir = HDirection::Left
        } else if pos.translation.x - RADIUS < -boxwidth / 2 as f32 {
            *hdir = HDirection::Right
        }

        if pos.translation.y + RADIUS > boxheight / 2 as f32 {
            *vdir = VDirection::Down
        } else if pos.translation.y - RADIUS < -boxheight / 2 as f32 {
            *vdir = VDirection::Up
        }
    }
}
