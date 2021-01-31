use bevy::prelude::*;
use bevy_rapier2d::physics::*;
use bevy_rapier2d::render::RapierRenderPlugin;
use bevy_rapier2d::rapier::{dynamics::*, geometry::ColliderBuilder};
use bevy_rapier2d::rapier::na::Vector2;

struct Player;

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>,
         mut rapier_config: ResMut<RapierConfiguration>,
         ) {

    rapier_config.scale = 20.0;
    commands.spawn(Camera2dBundle::default());

    let block_mat = materials.add(Color::rgb(0.0, 1.0, 0.0).into());
    let block = |cmd: &mut Commands, x: f32, y: f32, w: f32, h: f32| {
        cmd.spawn(SpriteBundle {
            material: block_mat.clone(),
            // transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
            sprite: Sprite::new(Vec2::new(w, h)),
            ..Default::default()
        })
        .with(RigidBodyBuilder::new_static().translation(x / rapier_config.scale, y / rapier_config.scale))
        .with(ColliderBuilder::cuboid(w / 2.0 / rapier_config.scale, h / 2.0 / rapier_config.scale));
    };

    let psize = 30.0;
    for i in 0..100 {
        let i = i as f32 / 100.0;
        commands.spawn(SpriteBundle {
            material: materials.add(Color::rgb(i, 1.0 - i, 1.0).into()),
            // transform: Transform::from_translation(Vec3::new(0.0, -415.0, 0.0)),
            sprite: Sprite::new(Vec2::new(psize, psize)),
            ..Default::default()
        })
            .with(RigidBodyBuilder::new_dynamic())
                .with(Player)
            .with(ColliderBuilder::cuboid(psize / 2.0 / rapier_config.scale, psize / 2.0/rapier_config.scale));
    }

    commands.with(Player);

    block(commands, 0.0, -200.0, 1000.0, 10.0);
    block(commands, 0.0, 200.0, 1000.0, 10.0);
    block(commands, -200.0, 0.0, 10.0, 1000.0);
    block(commands, 200.0, 0.0, 10.0, 1000.0);
}

#[derive(Default)]
struct Velocity(Vec2);

#[derive(Default)]
struct Force(Vec2);

fn physics( 
    time: Res<Time>, 
    mut query: Query<(&mut Transform, &mut Velocity, Option<&mut Force>)>
) {
    for (mut transform, mut vel, force) in query.iter_mut() {
        if let Some(mut force) = force {
            vel.0 += std::mem::take(&mut force.0);
        }
        transform.translation += vel.0.extend(0.0) * time.delta_seconds();
    }
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut query: Query<(&Player, &RigidBodyHandleComponent)>
) {
    for (_player, rb_comp) in query.iter_mut() {
        let mut force = Vector2::zeros();
        if keyboard_input.pressed(KeyCode::Up) {
            force.y += 5.0;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            force.y -= 5.0;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            force.x -= 5.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            force.x += 5.0;
        }
        let rb = rigid_bodies.get_mut(rb_comp.handle()).unwrap();
        rb.apply_impulse(force, true);
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin)
        // .add_plugin(RapierRenderPlugin)
        .add_startup_system(setup.system())
        .add_system(move_player.system())
        // .add_system(physics.system())
        .run();
}
