use std::{collections::HashMap, todo};

use bevy::prelude::*;
use bevy_rapier2d::{physics::*, rapier::{self, geometry::{ColliderHandle, ColliderSet, ContactEvent, InteractionGroups, NarrowPhase}, math::Point, parry::partitioning::IndexedData}};
use bevy_rapier2d::rapier::{dynamics::*, geometry::ColliderBuilder};
use bevy_rapier2d::rapier::na::Vector2;

/*
NOTES:
https://discord.com/channels/507548572338880513/748627261384556715
https://github.com/bevyengine/awesome-bevy
*/

struct Player {
    nextFire: f64,
}
struct Blob;
struct HitBox;

const WRLD_GRP: u16 = 0b1000000000000000;
const PLYR_GRP: u16 = 0b0100000000000000;
const BLOB_GRP: u16 = 0b0010000000000000;
const ALL_GRP: u16 = u16::MAX;

const SCALE: f32 = 20.0;
const SCALE2: f32 = 1.0; // 20.0;

#[derive(Bundle)]
struct BoxBundle {
    sprite_bundle: SpriteBundle,
    rb: RigidBodyBuilder,
    col: ColliderBuilder,
}

// struct BoxBundleBuilder {
//     x: f32,
//     y: f32,
//     w: f32,
//     h: f32,
//     material: Handle<ColorMaterial>,
//     scale: f32,
// }

// struct SpawnArgs<R, C> {
//     x: f32,
//     y: f32,
//     w: f32,
//     h: f32,
//     dynamic: bool,
//     rig_cb: R,
//     col_cb: C,
// }

// impl<R, C> Default for SpawnArgs<R, C> {
//     fn default
// }

fn spawnBox<'a>(
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
    let rb = if dynamic { RigidBodyBuilder::new_dynamic() } else { RigidBodyBuilder::new_static() }
        .translation(x / SCALE, y / SCALE)
        ;
    let rb = rig_cb(rb);
    let col = ColliderBuilder::cuboid(w / 2.0 / SCALE, h / 2.0 / SCALE)
        .user_data(ent.to_bits() as u128)
        ;
    let col = col_cb(col);
    cmd
        .with(rb)
        .with(col)
}

fn setup(
    commands: &mut Commands, 
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // global settings
    rapier_config.scale = SCALE;
    rapier_config.gravity = Vector2::new(0.0, -100.0 * SCALE2);

    // camera
    commands.spawn(Camera2dBundle::default());

    // world platforms
    let block_mat = materials.add(Color::rgba(0.0, 1.0, 0.0, 0.2).into());
    let block = |cmd: &mut Commands, x: f32, y: f32, w: f32, h: f32| {
        spawnBox(cmd,
            block_mat.clone(),
            x, y,
            w, h,
            false,
            |rig| {
                rig
            },
            |col| {
                col.collision_groups(InteractionGroups::new(WRLD_GRP, ALL_GRP))
            },
        );
    };
    block(commands, 0.0, -200.0, 2000.0, 100.0);
    block(commands, 0.0, 400.0, 2000.0, 100.0);
    block(commands, -600.0, 0.0, 100.0, 2000.0);
    block(commands, 600.0, 0.0, 100.0, 2000.0);

    // player
    let player_size = 20.0;
    spawnBox(commands,
        // Color::rgb(0.1, 0.9, 1.0),
        materials.add(Color::rgb(0.1, 0.9, 1.0).into()),
        0.0, 10.0,
        player_size, player_size,
        true,
        |rig| {
            rig.mass(1.0 * SCALE2)
        },
        |col| {
            col.collision_groups(InteractionGroups::new(PLYR_GRP, ALL_GRP))
        },
    ).with(Player {
        nextFire: 0.0
    });

    // blobs
    let blob_num = 4;
    let blob_size = 50.0;
    for x in 0..blob_num {
        for y in 0..blob_num {
            let c = (x + y*blob_num) as f32 / (blob_num*blob_num) as f32;
            let xf = x as f32;
            let yf = y as f32;
            let pnumf = blob_num as f32;
            let s = blob_size;
            spawnBox(commands,
                materials.add(Color::rgb(c, 1.0 - c, 1.0).into()),
                xf - pnumf * 0.5 + yf * 0.2 *s  /SCALE,(y - blob_num/2) as f32 *s /SCALE,
                s, s,
                true,
                |rig| {
                    rig.mass(0.1 * SCALE2)
                },
                |col| {
                    col.collision_groups(InteractionGroups::new(BLOB_GRP, ALL_GRP))
                        .friction(0.2)
                },
            )
                .with(Blob)
                ;
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
            if player.nextFire < time.seconds_since_startup() {
                player.nextFire = time.seconds_since_startup() + 1.0;
                
                let player_rb = rigid_bodies.get_mut(rb_comp.handle()).unwrap();
                let s = 10.0;
                let y = player_rb.position().translation.y * SCALE;
                let x = player_rb.position().translation.x * SCALE;
                let v = if player_rb.linvel().x > 0.0 {5.0} else {-5.0};

                spawnBox(commands,
                    materials.add(Color::rgb(1.0, 1.0, 0.8).into()),
                    x, y,
                    s, s,
                    true,
                    |rg| {
                        rg.gravity_scale(0.0)
                            .mass(10.0)
                            .linvel(player_rb.linvel().x + v, 0.0)
                    },
                    |col| {
                        col.collision_groups(InteractionGroups::new(PLYR_GRP, ALL_GRP & !PLYR_GRP))
                            // .sensor(true)
                    },
                )
                    .with(Despawn::after(1.0))
                    .with(HitBox)
                    .with(Collisions::default())
                    ;
            }
        }
    }
}

fn punch_hit(
    mut col_bodies: ResMut<ColliderSet>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    commands: &mut Commands,
    hitboxes: Query<(Entity, &RigidBodyHandleComponent, &ColliderHandleComponent), With<HitBox>>,
    blobs: Query<(Entity, &RigidBodyHandleComponent, &ColliderHandleComponent), With<Blob>>,
    narrow: NarrowPhase,
) {
    for (hb, hb_rb_comp, hb_col_comp) in hitboxes.iter() {
        let hb_rb = rigid_bodies.get_mut(hb_rb_comp.handle()).unwrap();
        for (blob, blob_rb_comp, blob_col_comp) in blobs.iter() {
            if let Some(true) = narrow.intersection_pair(hb_col_comp.handle(), blob_col_comp.handle()) {
                println!("POW!");
            }
        }
    }
}

fn print_events(
    events: Res<EventQueue>,
    mut col_bodies: ResMut<ColliderSet>,
    mut rigid_bodies: ResMut<RigidBodySet>,
) {
    // while let Ok(inter_event) = events.intersection_events.pop() {
    //     let col = col_bodies.get_mut(inter_event.collider1).unwrap();
    //     let rb = rigid_bodies.get_mut(col.parent()).unwrap();
    //     // col.parent()
    //     // println!("Received contact event: {:?}", inter_event);
    // }
}

#[derive(Default)]
struct Collisions(Vec<Entity>);
#[derive(Default)]
struct Intersections(Vec<Entity>);

fn find_collisions(
    events: Res<EventQueue>,
    mut handles: Query<(Entity, &ColliderHandleComponent)>,
    mut collisions: Query<&mut Collisions>,
    mut intersections: Query<&mut Intersections>,
) {
    let map: HashMap<ColliderHandle, _> = handles.iter().map(|(e, h)| {
        (h.handle(), e)
    }).collect();
    while let Ok(ContactEvent::Started(c1, c2)) = events.contact_events.pop() {
        if let (Some(&e1), Some(&e2)) = (map.get(&c1), map.get(&c2)) {
            collisions.get_mut(e1).map(|mut ids| ids.0.push(e2));
            collisions.get_mut(e2).map(|mut ids| ids.0.push(e1));
        }
    }
    while let Ok(inter) = events.intersection_events.pop() {
        if let (Some(&e1), Some(&e2)) = (map.get(&inter.collider1), map.get(&inter.collider2)) {
            intersections.get_mut(e1).map(|mut ids| ids.0.push(e2));
            intersections.get_mut(e2).map(|mut ids| ids.0.push(e1));
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

fn do_punch(
    commands: &mut Commands,
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut hitboxes: Query<(Entity, &Collisions), With<HitBox>>,
    mut blobs: Query<(Entity, &RigidBodyHandleComponent), With<Blob>>,
) {
    // build map of Entity -> blobs
    // let blob_map: HashMap<Entity, _> = blobs.iter().map(|tup| (tup.0, tup)).collect();
    for (hb_ent, collisions) in hitboxes.iter() {
        for &ent in collisions.0.iter() {
            if let Ok((_, blob_rb_comp)) = blobs.get(ent) {
                // hb collided with blob_ent
                // do punch
                let blob_rb = rigid_bodies.get_mut(blob_rb_comp.handle()).unwrap();
                blob_rb.apply_impulse(Vector2::new(0.0, 200.0), true);

                // despawn
                commands.despawn(hb_ent);
                break;
            }
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

        // constants
        let phys_scal = rb.mass() * 2.0;
        let angf_mag = 0.01 * phys_scal;
        let sidef_mag = 200.0 * phys_scal;
        let frig_s = 10.0 * phys_scal;
        let jump_mag = 15.0 * phys_scal;

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
        .add_system(player_shoot.system())
        .add_system(despawn_system.system())
        .add_system(print_events.system())
        .add_system_to_stage(stage::PRE_UPDATE, find_collisions.system())
        .add_system(do_punch.system())
        .add_system_to_stage(stage::LAST, clear_collisions.system())
        .run();
}
