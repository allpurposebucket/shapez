use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings, BloomPrefilterSettings},
        tonemapping::Tonemapping,
    }
};

use rand::prelude::*;

#[derive(Component)]
struct EnemySpawner {
    current_count: u32,
    max_count: u32,
}

#[derive(Component)]
struct Player {
    move_speed: f32,
    size: f32,
}

#[derive(Component)]
struct Enemy {
    velocity: Vec2,
    size: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, get_player_input)
        .add_systems(Update, update_player_size)
        .add_systems(Update, confine_player)
        .add_systems(Update, spawn_enemies)
        .add_systems(Update, move_enemies)
        .add_systems(Update, confine_enemies)
        .add_systems(Update, check_collisions)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings {
            intensity: 0.15,
            low_frequency_boost: 0.7,
            low_frequency_boost_curvature: 0.95,
            high_pass_frequency: 1.0,
            prefilter_settings: BloomPrefilterSettings {
                threshold: 0.6,
                threshold_softness: 0.2,
            },
            composite_mode: BloomCompositeMode::Additive,
        },
    ));

    commands.spawn((MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle { radius: 1.0 })),
        material: materials.add(Color::rgb(1.0, 3.0, 5.0)),
        transform: Transform::from_xyz(
            0.0,
            0.0,
            0.0,
        ),
        ..default()
    }, Player {move_speed: 200., size: 2.}));

    commands.spawn(EnemySpawner {
        current_count: 0,
        max_count: 10,
    });
}

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut enemy_spawner_q: Query<&mut EnemySpawner>,
    window_q: Query<&Window>,
) {
    let window = window_q.single();
    
    let mut enemy_spawner = enemy_spawner_q.single_mut();
    while enemy_spawner.current_count <= enemy_spawner.max_count {
        let random_x = rand::thread_rng().gen_range((-window.width() / 2.)..(window.width() / 2.));
        let random_y = rand::thread_rng().gen_range((-window.height() / 2.)..(window.height() / 2.));

        let random_size = random::<f32>() * 20.;

        let random_r = random::<f32>() * 2.;
        let random_g = random::<f32>() * 2.;
        let random_b = random::<f32>() * 2.;

        let random_velx = random::<f32>() * 10.;
        let random_vely = random::<f32>() * 10.;
        
        let random_velocity = Vec2 { x: random_velx, y: random_vely };

        commands.spawn((MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: random_size })),
            material: materials.add(Color::rgb(random_r, random_g, random_b)),
            transform: Transform::from_xyz(
                random_x,
                random_y,
                -1.,
            ),
            ..default()
        },
        Enemy {
            velocity: random_velocity,
            size: random_size,
        }
        ));
        enemy_spawner.current_count += 1;
    }
}

fn get_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_transform_q: Query<&mut Transform, With<Player>>,
    player_q: Query<&Player>,
) {
    let mut player_transform = player_transform_q.single_mut();
    let player = player_q.single();
    if keyboard_input.pressed(KeyCode::KeyW) {
        player_transform.translation.y += player.move_speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        player_transform.translation.y -= player.move_speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        player_transform.translation.x -= player.move_speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        player_transform.translation.x += player.move_speed * time.delta_seconds();
    }
}

fn move_enemies(
    mut enemy_q: Query<(&mut Transform, &Enemy), With<Enemy>>,
) {
    for (mut enemy_transform, enemy) in enemy_q.iter_mut() {
        enemy_transform.translation += Vec3 { x: enemy.velocity.x, y: enemy.velocity.y, z: 0. };
    }
}

fn update_player_size(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_transform_q: Query<&mut Transform, With<Player>>,
    mut player_q: Query<&mut Player>,
) {
    let mut player_transform = player_transform_q.single_mut();
    let mut player = player_q.single_mut();
    
    if keyboard_input.just_pressed(KeyCode::Space) {
        player_transform.scale *= 2.;
        player.size *= 2.;
    }
}

fn confine_player(
    window_q: Query<&Window>,
    mut player_transform_q: Query<&mut Transform, With<Player>>,
    player_q: Query<&Player>,
) {
    let window = window_q.single();
    let mut player_transform = player_transform_q.single_mut();
    let player = player_q.single();

    let half_player_size = player.size / 2.;
    
    let mut translation = player_transform.translation;

    let x_min = -(window.width() / 2.0) + half_player_size;
    let x_max = window.width() / 2.0 - half_player_size;
    let y_min = -(window.height() / 2.0) + half_player_size;
    let y_max = window.height() / 2.0 - half_player_size;
    
    if translation.x < x_min {
        translation.x = x_min;
    }
    else if translation.x > x_max {
        translation.x = x_max;
    }
    if translation.y < y_min {
        translation.y = y_min;
    }
    else if translation.y > y_max {
        translation.y = y_max;
    }

    player_transform.translation = translation;
}

fn confine_enemies(
    window_q: Query<&Window>,
    mut enemy_q: Query<(&mut Enemy, &Transform), With<Enemy>>,
) {
    let window = window_q.single();

    for (mut enemy, enemy_transform) in enemy_q.iter_mut() {
        let half_enemy_size = enemy.size / 2.;
        let translation = enemy_transform.translation;
        
        let x_min = -(window.width() / 2.0) + half_enemy_size;
        let x_max = window.width() / 2.0 - half_enemy_size;
        let y_min = -(window.height() / 2.0) + half_enemy_size;
        let y_max = window.height() / 2.0 - half_enemy_size;

        if translation.x < x_min || translation.x > x_max {
            enemy.velocity.x = -enemy.velocity.x;
        }
        if translation.y < y_min || translation.y > y_max {
            enemy.velocity.y = -enemy.velocity.y;
        }
    }
}

fn check_collisions(
    player_q: Query<(&Player, &Transform), With<Player>>,
    enemy_q: Query<(&Enemy, &Transform), With<Enemy>>,
) {
    let (player, player_transform) = player_q.single();

    for (enemy, enemy_transform) in enemy_q.iter() {
        let x_min = enemy_transform.translation.x - (enemy.size / 2.) - (player.size / 2.);
        let x_max = enemy_transform.translation.x + (enemy.size / 2.) + (player.size / 2.);
        let y_min = enemy_transform.translation.y - (enemy.size / 2.) - (player.size / 2.);
        let y_max = enemy_transform.translation.y + (enemy.size / 2.) + (player.size / 2.);
        
        if player_transform.translation.x > x_min && player_transform.translation.x < x_max && player_transform.translation.y > y_min && player_transform.translation.y < y_max  {
            println!("Collision.");
        }
    }
}
