use std::todo;

use bevy::prelude::*;
use bevy_rapier2d::{physics::*, rapier::geometry::InteractionGroups};
use bevy_rapier2d::rapier::{dynamics::*, geometry::ColliderBuilder};
use bevy_rapier2d::rapier::na::Vector2;

struct Player;
struct Blob;

const WRLD_GRP: u16 = 0b1000000000000000;
const PLYR_GRP: u16 = 0b0100000000000000;
const BLOB_GRP: u16 = 0b0010000000000000;
const ALL_GRP: u16 = u16::MAX;

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>,
         mut rapier_config: ResMut<RapierConfiguration>,
         ) {

    rapier_config.scale = 20.0;
    rapier_config.gravity = Vector2::new(0.0, -100.0);
    commands.spawn(Camera2dBundle::default());

    let block_mat = materials.add(Color::rgba(0.0, 1.0, 0.0, 0.2).into());
    let block = |cmd: &mut Commands, x: f32, y: f32, w: f32, h: f32| {
        cmd.spawn(SpriteBundle {
            material: block_mat.clone(),
            // transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
            sprite: Sprite::new(Vec2::new(w, h)),
            ..Default::default()
        })
        .with(RigidBodyBuilder::new_static().translation(x / rapier_config.scale, y / rapier_config.scale))
        .with(ColliderBuilder::cuboid(w / 2.0 / rapier_config.scale, h / 2.0 / rapier_config.scale)
            .collision_groups(InteractionGroups::new(WRLD_GRP, ALL_GRP))
        )   
        // .with(Msaa::default())
        ;
    };

    let player_size = 20.0;
    commands.spawn(SpriteBundle {
        material: materials.add(Color::rgb(0.1, 0.9, 1.0).into()),
        sprite: Sprite::new(Vec2::new(player_size, player_size)),
        ..Default::default()
    })
        .with(RigidBodyBuilder::new_dynamic()
            .mass(1.0)
            .translation(0.0, 10.0))
        .with(Player)
        .with(ColliderBuilder::cuboid(player_size / 2.0 / rapier_config.scale, player_size / 2.0/rapier_config.scale)
            .collision_groups(InteractionGroups::new(PLYR_GRP, ALL_GRP))
        );

    let pnum = 4;
    let psize = 50.0;
    for x in 0..pnum {
        for y in 0..pnum {
            let c = (x + y*pnum) as f32 / (pnum*pnum) as f32;
            let xf = x as f32;
            let yf = y as f32;
            let pnumf = pnum as f32;
            // let s = psize + psize * c * 2.0;
            let s = psize;
            commands.spawn(SpriteBundle {
                material: materials.add(Color::rgb(c, 1.0 - c, 1.0).into()),
                // transform: Transform::from_translation(Vec3::new(0.0, -415.0, 0.0)),
                sprite: Sprite::new(Vec2::new(s, s)),
                ..Default::default()
            })
                .with(RigidBodyBuilder::new_dynamic()
                    .mass(0.1)
                    .translation(xf - pnumf * 0.5 + yf * 0.2 *s  /rapier_config.scale,(y - pnum/2) as f32 *s /rapier_config.scale))
                .with(Blob)
                .with(
                    ColliderBuilder::cuboid(s / 2.0 / rapier_config.scale, s / 2.0/rapier_config.scale)
                    .friction(0.2)
                    .collision_groups(InteractionGroups::new(BLOB_GRP, ALL_GRP))
                );
        }
    }
    block(commands, 0.0, -200.0, 2000.0, 100.0);
    block(commands, 0.0, 400.0, 2000.0, 100.0);
    block(commands, -600.0, 0.0, 100.0, 2000.0);
    block(commands, 600.0, 0.0, 100.0, 2000.0);
}

#[derive(Default)]
struct Velocity(Vec2);

#[derive(Default)]
struct Force(Vec2);

fn player_fight(
    keyboard_input: Res<Input<KeyCode>>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    query: Query<(Entity, &RigidBodyHandleComponent), With<Player>>
) {
    for (player, rb_comp) in query.iter() {
        if keyboard_input.pressed(KeyCode::Space) {
            let player_rb = rigid_bodies.get_mut(rb_comp.handle()).unwrap();

            // side_force.x -= sidef_mag
            let s = 10.0;
            let y = player_rb.position().translation.y;
            let x = player_rb.position().translation.x;
            
            let v = if player_rb.linvel().x > 0.0 {5.0} else {-5.0};

            let rb = RigidBodyBuilder::new_dynamic()
                .gravity_scale(0.0)
                .mass(10.0)
                // TODO @darzu: translate to player
                .translation(x, y)
                .linvel(player_rb.linvel().x + v, 0.0);
            // rb.apply_impulse(punch_force, true);

            commands.spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 0.8).into()),
                // transform: Transform::from_translation(Vec3::new(-x, -y, 0.0)),
                transform: Transform::from_translation(Vec3::new(x * 20.0, y * 20.0, 0.0)),
                // transform: Transform::from_translation(Vec3::new(-x * 20.0, -y * 20.0, 0.0)),
                sprite: Sprite::new(Vec2::new(s, s)),
                ..Default::default()
            })
                // .with(Parent(player))
                .with(rb)
                .with(Blob)
                .with(
                    ColliderBuilder::cuboid(s / 2.0 / rapier_config.scale, s / 2.0/rapier_config.scale)
                    .collision_groups(InteractionGroups::new(PLYR_GRP, ALL_GRP & !PLYR_GRP))
                    // .sensor(true)
                )
                .with(Despawn::after(1.0))
                ;
        }
    }
}

fn player_move(
    keyboard_input: Res<Input<KeyCode>>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    query: Query<(&Player, &RigidBodyHandleComponent)>
) {
    for (_player, rb_comp) in query.iter() {
        let rb = rigid_bodies.get_mut(rb_comp.handle()).unwrap();

        let phys_scal = rb.mass() * 2.0;

        let mut vel = Vector2::new(rb.linvel().x, rb.linvel().y);
        let mut jump_force = Vector2::zeros();
        let mut fric_force = Vector2::zeros();
        let mut side_force = Vector2::zeros();
        let mut ang_vel = rb.angvel();
        let angf_mag = 0.01 * phys_scal;
        let sidef_mag = 200.0 * phys_scal;
        let frig_s = 10.0 * phys_scal;
        let jump_mag = 15.0 * phys_scal;
        if keyboard_input.just_pressed(KeyCode::W) {
            jump_force.y += jump_mag;
        }
        if keyboard_input.pressed(KeyCode::A) {
            side_force.x -= sidef_mag
        }
        if keyboard_input.pressed(KeyCode::D) {
            side_force.x += sidef_mag
        }
        if keyboard_input.pressed(KeyCode::Q) {
            ang_vel += angf_mag
        }
        else if keyboard_input.pressed(KeyCode::E) {
            ang_vel -= angf_mag
        }
        // else {
        //     ang_vel = 0.0;
        // }
        if jump_force.magnitude_squared() > 0.0 {
            // reset y vel when jumping
            vel.y = 0.0;
        }
        rb.set_linvel(vel, true);
        rb.set_angvel(ang_vel, true);
        if jump_force.magnitude_squared() > 0.0 {
            rb.apply_impulse(jump_force, true);
        }
        if side_force.magnitude_squared() > 0.0 {
            rb.apply_force(side_force, true);
        }
        fric_force.x = -rb.linvel().x*frig_s;
        if fric_force.magnitude_squared() > 0.0 {
            rb.apply_force(fric_force, true);
        }
    }
}

struct Despawn {
    timer: Timer
}

impl Despawn {
    fn after(time: f32) -> Self {
        Self { timer: Timer::from_seconds(time, false) }
    }
}

fn despawn_system(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Despawn)>,
) {
    for (entity, mut despawn) in query.iter_mut() {
        if despawn.timer.tick(time.delta_seconds()).just_finished() {
            commands.despawn(entity);
        }
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin)
        // .add_plugin(RapierRenderPlugin)
        .add_startup_system(setup.system())
        .add_system(player_move.system())
        .add_system(player_fight.system())
        .add_system(despawn_system.system())
        // .add_system(physics.system())
        .run();
}
