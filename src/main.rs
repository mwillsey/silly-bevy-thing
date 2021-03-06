use std::{borrow::Borrow, collections::HashMap};

use bevy::{math::vec2, prelude::*, render::camera::Camera};
use bevy_rapier2d::rapier::{geometry::{Collider, ColliderSet}, na::Vector2};
use bevy_rapier2d::rapier::{dynamics::*, geometry::ColliderBuilder};
use bevy_rapier2d::{
    physics::*,
    rapier::geometry::{ColliderHandle, ContactEvent, InteractionGroups, NarrowPhase},
};

/*
NOTES:
https://discord.com/channels/507548572338880513/748627261384556715
https://github.com/bevyengine/awesome-bevy
*/

struct Player {
    next_fire: f64,
    dir_x: f32,
}
#[derive(Default)]
struct Blob {
    going_right: bool
}
struct Platform;
struct HitBox {
    dir_x: f32,
}

const WRLD_GRP: u16 = 0b1000000000000000;
const PLYR_GRP: u16 = 0b0100000000000000;
const BLOB_GRP: u16 = 0b0010000000000000;
const ALL_GRP: u16 = u16::MAX;

const SCALE: f32 = 20.0;

#[derive(Bundle)]
struct BoxBundle {
    sprite_bundle: SpriteBundle,
    rb: RigidBodyBuilder,
    col: ColliderBuilder,
}

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
    .can_sleep(false)
    .translation(x / SCALE, y / SCALE);
    let rb = rig_cb(rb);
    let col =
        ColliderBuilder::cuboid(w / 2.0 / SCALE, h / 2.0 / SCALE)
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
            col.collision_groups(InteractionGroups::new(BLOB_GRP, ALL_GRP))
                .friction(0.2)
        },
    )
    .with(Blob::default())
    .with(Intersections::default())
    .with(Health { health: 10 })
}

fn gen_world(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let block_mat = materials.add(Color::rgba(0.0, 1.0, 0.0, 0.2).into());
    let block = |cmd: &mut Commands, x: f32, y: f32, w: f32, h: f32| {
        spawn_box(
            cmd,
            block_mat.clone(),
            x,
            y,
            w,
            h,
            false,
            |rig| rig,
            |col| col.collision_groups(InteractionGroups::new(WRLD_GRP, ALL_GRP)),
        )
        .with(Platform);
    };

    let S = 2;
    for x in -S..S {
        for y in -S..S {
            // spawn platform
            let (x, y) = (x as f32, y as f32);
            let (w, h) = (200.0, 10.0);
            let xx = x * w * 2.0 + y * w;
            let yy = y * 100.0;
            block(commands, xx, yy, w, h);

            // spawn blob?
            let c = 0.5;
            spawn_blob(commands, materials, 
                xx, 
                yy + 5.0,
                c);
        }
    }
}

fn move_camera(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let player_tf = player_query.iter().next().unwrap();
    let mut camera_tf = camera_query.iter_mut().next().unwrap();
    
    let diff = player_tf.translation - camera_tf.translation;
    camera_tf.translation += diff * 0.01;
}

fn setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    println!("Hello world!");

    // global settings
    rapier_config.scale = SCALE;
    rapier_config.gravity = Vector2::new(0.0, -100.0);

    // camera
    commands.spawn(Camera2dBundle::default());

    // world platforms
    gen_world(commands, &mut materials);

    // player
    let player_size = 20.0;
    spawn_box(
        commands,
        // Color::rgb(0.1, 0.9, 1.0),
        materials.add(Color::rgb(0.2, 0.9, 0.8).into()),
        0.0,
        10.0,
        player_size,
        player_size,
        true,
        |rig| rig.mass(1.0),
        |col| col.collision_groups(InteractionGroups::new(PLYR_GRP, ALL_GRP)),
    )
    .with(Player { 
        next_fire: 0.0, 
        dir_x: 1.0 
    });

    // blobs
    // let blob_num = 4;
    // let blob_size = 50.0;
    // for x in 0..blob_num {
    //     for y in 0..blob_num {
    //         let c = (x + y * blob_num) as f32 / (blob_num * blob_num) as f32;
    //         let xf = x as f32;
    //         let yf = y as f32;
    //         let pnumf = blob_num as f32;
    //         let s = blob_size;
    //         spawn_blob(commands, &mut materials, 
    //             xf - pnumf * 0.5 + yf * 0.2 * s / SCALE, 
    //             (y - blob_num / 2) as f32 * s / SCALE,
    //             c);
    //     }
    // }
}

fn other<T>(me: ColliderHandle, contact: (ColliderHandle, ColliderHandle, T)) -> ColliderHandle {
    if me == contact.0 {
        contact.1
    } else {
        assert_eq!(me, contact.1);
        contact.0
    }
}

fn bool2f(b: bool) -> f32 {
    if b { 1.0 } else { -1.0 }
}

fn blob_move(
    narrow_phase: Res<NarrowPhase>,
    colliders: ResMut<ColliderSet>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut blobs: Query<(&mut Blob, &RigidBodyHandleComponent, &ColliderHandleComponent)>,
    platforms: Query<&ColliderHandleComponent, With<Platform>>,
) {
    for (mut blob, blob_rbh, blob_cth) in blobs.iter_mut() {
        let blob_cth = blob_cth.handle();
        if let Some(contacting) = narrow_phase.contacts_with(blob_cth) {
            let contacting_platforms: Vec<_> = contacting.filter_map(|ct| {
                let other_ent = c2e(&colliders[other(blob_cth, ct)]);
                platforms.get(other_ent).ok()
            }).collect();
            if contacting_platforms.len() == 1 {
                let blob_rb = &mut rigid_bodies[blob_rbh.handle()];
                if blob_rb.linvel().y.abs() <= 0.01 {
                   let plat_aabb = colliders[contacting_platforms[0].handle()].compute_aabb();
                   let blob_aabb = colliders[blob_cth].compute_aabb();
                   let x = 30.0 * if blob.going_right {
                       bool2f(blob_aabb.maxs.x < plat_aabb.maxs.x)
                   } else {
                       -bool2f(blob_aabb.mins.x > plat_aabb.mins.x)
                   };
                   blob.going_right = x.is_sign_positive();
                   blob_rb.apply_impulse([x, 100.0].into(), true);
                }
            }
        }
    }
}

#[derive(Default)]
struct Velocity(Vec2);

#[derive(Default)]
struct Force(Vec2);

fn player_shoot(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut Player, &RigidBodyHandleComponent)>,
) {
    for (mut player, rb_comp) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::F) {
            if player.next_fire < time.seconds_since_startup() {
                player.next_fire = time.seconds_since_startup() + 0.5;

                let player_rb = rigid_bodies.get_mut(rb_comp.handle()).unwrap();
                let s = 40.0;
                let y = player_rb.position().translation.y * SCALE;
                let x = player_rb.position().translation.x * SCALE;
                let x_off = (s*0.5 + 10.0) * player.dir_x;
                // let v = if player_rb.linvel().x > 0.0 {
                //     5.0
                // } else {
                //     -5.0
                // };

                spawn_box(
                    commands,
                    materials.add(Color::rgba(1.0, 1.0, 1.0, 0.1).into()),
                    x + x_off,
                    y + 10.0*1.5,
                    s,
                    s,
                    true,
                    |rg| {
                        rg.gravity_scale(0.0)
                            // .mass(10.0)
                            // .linvel(player_rb.linvel().x + v, 0.0)
                    },
                    |col| {
                        col.collision_groups(InteractionGroups::new(PLYR_GRP, ALL_GRP & !PLYR_GRP))
                            .sensor(true)
                    },
                )
                .with(Despawn::after(0.1))
                .with(HitBox { dir_x: player.dir_x })
                .with(Intersections::default());
            }
        }
    }
}

#[derive(Default)]
struct Collisions(Vec<Entity>);
#[derive(Default)]
struct Intersections(Vec<Entity>);

fn find_collisions(
    events: Res<EventQueue>,
    handles: Query<(Entity, &ColliderHandleComponent)>,
    mut collisions: Query<&mut Collisions>,
    mut intersections: Query<&mut Intersections>,
) {
    let map: HashMap<ColliderHandle, _> = handles.iter().map(|(e, h)| (h.handle(), e)).collect();
    while let Ok(ContactEvent::Started(c1, c2)) = events.contact_events.pop() {
        if let (Some(&e1), Some(&e2)) = (map.get(&c1), map.get(&c2)) {
            if let Ok(mut ids) = collisions.get_mut(e1) {
                ids.0.push(e2)
            }
            if let Ok(mut ids) = collisions.get_mut(e2) {
                ids.0.push(e1)
            }
        }
    }
    while let Ok(inter) = events.intersection_events.pop() {
        if let (Some(&e1), Some(&e2)) = (map.get(&inter.collider1), map.get(&inter.collider2)) {
            if let Ok(mut ids) = intersections.get_mut(e1) {
                ids.0.push(e2)
            }
            if let Ok(mut ids) = intersections.get_mut(e2) {
                ids.0.push(e1)
            }
        }
    }
}

fn clear_collisions(
    mut collisions: Query<&mut Collisions>,
    mut intersections: Query<&mut Intersections>,
) {
    for mut c in collisions.iter_mut() {
        c.0.clear()
    }
    for mut i in intersections.iter_mut() {
        i.0.clear()
    }
}

// macro_rules! subquery {
//     // ($ids:expr, $query:expr) => {
//     //     $ids.iter().filter_map(|id| $query.get(*id).ok())
//     // };
//     ($ids:expr, mut $query:expr) => {
//         $ids.iter().filter_map(|id| $query.get_mut(*id).ok())
//     };
// }

fn do_punch(
    commands: &mut Commands,
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    // mut col_bodies: ResMut<ColliderSet>,
    hitboxes: Query<(Entity, &Intersections, &HitBox)>,
    mut blobs: Query<(Entity, &RigidBodyHandleComponent, &mut Health, &Handle<ColorMaterial>), With<Blob>>,
) {
    // build map of Entity -> blobs
    for (hb_ent, collisions, hb) in hitboxes.iter() {
        //(_, blob_rb_comp, blob)
        for &id in collisions.0.iter() {
            if let Ok((_, blob_rb_comp, mut blob_health, blob_color_comp)) = blobs.get_mut(id) {
                // hb collided with blob_ent
                // let hb_col = col_bodies.get_mut(hb_col_comp.handle()).unwrap();
                // let hb_col = col_bodies.get_mut(hb_col_comp.handle()).unwrap();
                // hb_col.
                
                // do punch
                let blob_rb = rigid_bodies.get_mut(blob_rb_comp.handle()).unwrap();
                // if let cub hb_col.shape().
                
                blob_rb.apply_impulse(Vector2::new(hb.dir_x * 50.0, 200.0), true);
                blob_health.health -= 1;

                let hf = blob_health.health as f32 / 10.0;

                let prev_color = materials.get(blob_color_comp).unwrap().color.clone();

                materials.set(blob_color_comp, Color::rgba(prev_color.r(), prev_color.g(), prev_color.b(), hf).into());

                // blob_color 
                // materials.add(Color::rgb(c, 1.0 - c, 1.0).into()),

                // despawn
                commands.despawn(hb_ent);
            }

        }
    }
}

fn player_move(
    keyboard_input: Res<Input<KeyCode>>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut query: Query<(&mut Player, &RigidBodyHandleComponent)>,
) {
    for (mut player, rb_comp) in query.iter_mut() {
        let rb = rigid_bodies.get_mut(rb_comp.handle()).unwrap();

        // constants
        let phys_scal = rb.mass() * 2.0;
        let angf_mag = 0.01 * phys_scal;
        let sidef_mag = 200.0 * phys_scal;
        let frig_s = 10.0 * phys_scal;
        let jump_mag = 20.0 * phys_scal;

        // forces, impulses and velocity updates
        let mut vel = Vector2::new(rb.linvel().x, rb.linvel().y);
        let mut jump_force = Vector2::zeros();
        let mut fric_force = Vector2::zeros();
        let mut side_force = Vector2::zeros();
        let mut ang_vel = rb.angvel();
        if keyboard_input.just_pressed(KeyCode::W) || keyboard_input.just_pressed(KeyCode::Space) {
            jump_force.y += jump_mag;
        }
        if keyboard_input.pressed(KeyCode::A) {
            side_force.x -= sidef_mag;
            player.dir_x = -1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            side_force.x += sidef_mag;
            player.dir_x = 1.0;
        }
        if keyboard_input.pressed(KeyCode::Q) {
            ang_vel += angf_mag
        } else if keyboard_input.pressed(KeyCode::E) {
            ang_vel -= angf_mag
        }
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
        fric_force.x = -rb.linvel().x * frig_s;
        if fric_force.magnitude_squared() > 0.0 {
            rb.apply_force(fric_force, true);
        }
    }
}

struct Despawn {
    timer: Timer,
}

impl Despawn {
    fn after(time: f32) -> Self {
        Self {
            timer: Timer::from_seconds(time, false),
        }
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

struct Health {
    health: u32,
}

// impl Health {
//     fn after(time: f32) -> Self {
//         Self {
//             timer: Timer::from_seconds(time, false),
//         }
//     }
// }

fn health_system(
    commands: &mut Commands,
    query: Query<(Entity, &Health)>,
) {
    for (entity, health) in query.iter() {
        if health.health <= 0 {
            commands.despawn(entity);
        }
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin)
        .add_startup_system(setup.system())
        .add_system(player_move.system())
        .add_system(blob_move.system())
        .add_system(player_shoot.system())
        .add_system(despawn_system.system())
        .add_system(health_system.system())
        .add_system(move_camera.system())
        .add_system_to_stage(stage::PRE_UPDATE, find_collisions.system())
        .add_system(do_punch.system())
        .add_system_to_stage(stage::LAST, clear_collisions.system())
        .run();
}
