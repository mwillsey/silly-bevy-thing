use std::{borrow::Borrow, collections::HashMap};
use bevy::{math::vec2, prelude::*, render::camera::Camera};
use bevy_rapier2d::rapier::{geometry::{Collider, ColliderSet}, na::Vector2};
use bevy_rapier2d::rapier::{dynamics::*, geometry::ColliderBuilder};
use bevy_rapier2d::{
    physics::*,
    rapier::geometry::{ColliderHandle, ContactEvent, InteractionGroups, NarrowPhase},
};

// types
#[derive(Default)]
struct Blob;
struct Platform;
#[derive(Default)]
struct Velocity(Vec2);
#[derive(Default)]
struct Force(Vec2);
#[derive(Bundle)]
struct BoxBundle {
    sprite_bundle: SpriteBundle,
    rb: RigidBodyBuilder,
    col: ColliderBuilder,
}

// constants
const ALL_GRP: u16 = u16::MAX;
const RAPIER_SCALE: f32 = 20.0;

// PHYSICS BUG
fn blob_move(
    time: Res<Time>,
    narrow_phase: Res<NarrowPhase>,
    colliders: ResMut<ColliderSet>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut blobs: Query<(&mut Blob, &RigidBodyHandleComponent, &ColliderHandleComponent)>,
    platforms: Query<&RigidBodyHandleComponent, With<Platform>>,
) {
    for (blob, blob_rbh, blob_cth) in blobs.iter_mut() {
        let blob_cth = blob_cth.handle();
        if let Some(contacting) = narrow_phase.contacts_with(blob_cth) {
            let contacting_platforms: Vec<_> = contacting.filter_map(|ct| {
                let other_ent = c2e(&colliders[other(blob_cth, ct)]);
                platforms.get(other_ent).ok()
            }).collect();
            if contacting_platforms.len() == 1 {
                let blob_rb = &mut rigid_bodies[blob_rbh.handle()];
                if !blob_rb.is_moving() /*|| true*/ { // NOTE: adding `|| true` here causes it not to crash.
                    // ---
                    // CRASH! This line. Comment out to hide crash
                    blob_rb.apply_impulse([100.0, 100.0].into(), true);
                    // CRASH! This line. Comment out to hide crash
                    // ---
                }
            }
        }
    }
}

fn other<T>(me: ColliderHandle, contact: (ColliderHandle, ColliderHandle, T)) -> ColliderHandle {
    if me == contact.0 {
        contact.1
    } else {
        assert_eq!(me, contact.1);
        contact.0
    }
}

// GAME SETUP
fn spawn_box<'a>(
    cmd: &'a mut Commands,
    material: Handle<ColorMaterial>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    dynamic: bool,
    rig_cb: impl Fn(RigidBodyBuilder) -> RigidBodyBuilder,
    col_cb: impl Fn(ColliderBuilder) -> ColliderBuilder,
) -> &'a mut Commands {
    let cmd = cmd.spawn(SpriteBundle {
        material: material,
        sprite: Sprite::new(Vec2::new(w, h)),
        transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
        ..Default::default()
    });
    let ent = cmd.current_entity().unwrap();
    let rb = if dynamic {
        RigidBodyBuilder::new_dynamic()
    } else {
        RigidBodyBuilder::new_static()
    }
    .translation(x / RAPIER_SCALE, y / RAPIER_SCALE);
    let rb = rig_cb(rb);
    let col =
        ColliderBuilder::cuboid(w / 2.0 / RAPIER_SCALE, h / 2.0 / RAPIER_SCALE)
          .user_data(ent.to_bits() as u128);
    let col = col_cb(col);
    cmd.with(rb).with(col)
}

fn c2e(c: &Collider) -> Entity {
    Entity::from_bits(c.user_data as u64)
}

fn spawn_blob<'a>(
    commands: &'a mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    x: f32, y: f32, c: f32,
) -> &'a mut Commands {    
    let blob_size = 50.0;
    spawn_box(
        commands,
        materials.add(Color::rgb(c, 1.0 - c, 1.0).into()),
        // materials.add(Color::rgb(0.0, 1.0, 1.0).into()),
        x,
        y,
        blob_size,
        blob_size,
        true,
        |rig| rig.mass(0.1),
        |col| {
            col.collision_groups(InteractionGroups::new(ALL_GRP, ALL_GRP))
                .friction(0.2)
        },
    )
    .with(Blob::default())
}

fn setup_game(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    println!("Hello world!");

    // global settings
    rapier_config.scale = RAPIER_SCALE;
    rapier_config.gravity = Vector2::new(0.0, -100.0);

    // camera
    commands.spawn(Camera2dBundle::default());

    // world platforms
    let block_mat = materials.add(Color::rgba(0.0, 1.0, 0.0, 0.2).into());
    let spawn_block = |cmd: &mut Commands, x: f32, y: f32, w: f32, h: f32| {
        spawn_box(
            cmd,
            block_mat.clone(),
            x,
            y,
            w,
            h,
            false,
            |rig| rig,
            |col| col.collision_groups(InteractionGroups::new(ALL_GRP, ALL_GRP)),
        )
        .with(Platform);
    };

    // spawn platform
    let (w, h) = (200.0, 10.0);
    spawn_block(commands, 0.0, 0.0, w, h);

    // spawn blob?
    spawn_blob(commands, &mut materials, 0.0, 5.0, 0.5);
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin)
        .add_startup_system(setup_game.system())
        .add_system(blob_move.system())
        .run();
}
