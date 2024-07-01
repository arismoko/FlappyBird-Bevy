use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy_prototype_lyon::draw::Fill;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::geometry::GeometryBuilder;
use bevy_prototype_lyon::shapes;
use bevy_xpbd_2d::prelude::*;
use rand::Rng;

#[derive(Event)]
pub struct PlayAudio(pub String);

#[derive(Component)]
pub struct Bird;

#[derive(Component)]
pub struct Pipe;

#[derive(Component)]
pub struct Cloud;

#[derive(Component)]
pub struct Building;

#[derive(Resource)]
pub struct Score(pub i32);

#[derive(Resource)]
pub struct LastPipeSpawnTime(pub f64);

#[derive(Resource)]
pub struct Gravity(pub f32);

#[derive(Bundle)]
pub struct MovableEntity {
    pub sprite_renderer: SpriteBundle,
    pub collider: Collider,
    pub velocity: LinearVelocity,
}
pub fn animate_clouds(mut query: Query<(&mut Transform, &Cloud)>, time: Res<Time>) {
    for (mut transform, _cloud) in query.iter_mut() {
        transform.translation.x -= time.delta_seconds() * 100.0;
        if transform.translation.x < -670.0 {
            transform.translation.x = 670.0;
        }
    }
}

pub fn animate_buildings(mut query: Query<(&mut Transform, &Building)>, time: Res<Time>) {
    for (mut transform, _building) in query.iter_mut() {
        transform.translation.x -= time.delta_seconds() * 50.0; // Buildings move slower than clouds
        if transform.translation.x < -800.0 {
            transform.translation.x = 800.0;
        }
    }
}

pub fn spawn_clouds(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let cloud_colors = [Color::rgb(1.0, 1.0, 1.0), Color::rgb(0.7, 0.7, 0.7)];

    for _ in 0..100 {
        let radius_a = rng.gen_range(20.0..100.0);
        let radius_b = rng.gen_range(1.0..100.0);

        let shape = shapes::Ellipse {
            radii: Vec2::new(radius_a, radius_b),
            center: Vec2::ZERO,
        };
        let x_position = rng.gen_range(-670.0..670.0);
        let y_position = rng.gen_range(350.0..400.0);
        let z_position = rng.gen_range(1.0..1.5); // Ensure unique Z values
        let index = ((z_position as f32 * 2.0).clamp(0.0, 0.5) * (cloud_colors.len() as f32))
            .round() as usize;
        let color = cloud_colors[cloud_colors.len() - 1 - index]; // Use Z position to determine color (lower Z = darker color)
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                spatial: SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(
                        x_position, y_position, z_position,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
            Fill::color(color),
            Cloud,
        ));
    }
}

pub fn spawn_buildings(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let building_colors = [
        Color::rgb(0.1, 0.1, 0.1),
        Color::rgb(0.2, 0.2, 0.2),
        Color::rgb(0.3, 0.3, 0.3),
    ];

    for _ in 0..20 {
        let width = rng.gen_range(50.0..200.0);
        let height = rng.gen_range(100.0..400.0);
        let color = building_colors[rng.gen_range(0..building_colors.len())];
        let shape = shapes::Rectangle {
            extents: Vec2::new(width, height),
            origin: shapes::RectangleOrigin::BottomLeft,
        };
        let x_position = rng.gen_range(-640.0..640.0);
        let y_position = -375.0;
        let z_position = rng.gen_range(0.0..0.5); // Ensure buildings are behind other elements

        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                spatial: SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(
                        x_position, y_position, z_position,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
            Fill::color(color),
            Building,
        ));
    }
}
pub fn bird_controller(
    mut query: Query<(&mut LinearVelocity, &mut Transform), With<Bird>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut audio_event_writer: EventWriter<PlayAudio>,
    mut gravity: ResMut<Gravity>,
) {
    for (mut bird_velocity, mut bird_transform) in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            gravity.0 = 0.0;
            bird_velocity.0.y = 150.;
            audio_event_writer.send(PlayAudio("sounds/jump.ogg".to_string()));
        }

        // Apply gravity
        gravity.0 -= 5.0 * time.delta_seconds(); // Reduced gravity to slow the fall
        bird_velocity.0.y += gravity.0;

        // Update position
        bird_transform.translation.y += bird_velocity.0.y * time.delta_seconds();
        bird_transform.translation.x += bird_velocity.0.x * time.delta_seconds();
    }
}

pub fn spawn_pipes(
    mut commands: Commands,
    time: Res<Time>,
    mut last_pipe_spawn_time: ResMut<LastPipeSpawnTime>,
) {
    let pipe_spawn_interval = 5.0; // seconds

    if time.elapsed_seconds_f64() - last_pipe_spawn_time.0 > pipe_spawn_interval {
        last_pipe_spawn_time.0 = time.elapsed_seconds_f64();

        let mut rng = rand::thread_rng();
        let pipe_gap = 175.0;
        let pipe_y = rng.gen_range(-200.0..200.0);

        let pipe_width = 100.0;
        let pipe_height = 670.0;

        // Upper pipe
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.0, 1.0, 0.0),
                    custom_size: Some(Vec2::new(pipe_width, pipe_height)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(800.0, pipe_y + pipe_gap / 2.0 + pipe_height / 2.0, 1.0),
                    ..default()
                },
                ..default()
            },
            Collider::rectangle(pipe_width, pipe_height),
            LinearVelocity(Vec2::new(-100.0, 0.0)),
            Pipe,
        ));

        // Lower pipe
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.0, 1.0, 0.0),
                    custom_size: Some(Vec2::new(pipe_width, pipe_height)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(800.0, pipe_y - pipe_gap / 2.0 - pipe_height / 2.0, 1.),
                    ..default()
                },
                ..default()
            },
            Collider::rectangle(pipe_width, pipe_height),
            LinearVelocity(Vec2::new(-100.0, 0.0)),
            Pipe,
        ));
    }
}

pub fn game_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Blue background
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.0, 0.0, 1.0),
            ..default()
        },
        transform: Transform {
            scale: Vec3::new(1280.0, 720.0, 1.0),
            translation: Vec3::new(0.0, 0.0, -1.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Score: 0".to_string(),
                style: TextStyle {
                    font_size: 40.0,
                    color: Color::BLACK,
                    ..default()
                },
            }],
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0., 100.0, 3.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let bird_texture_handle = asset_server.load("rustacean-flat-happy.png");
    let bird_size = Vec2::new(460.0, 307.0); // Assuming the sprite is 256x256
    let bird_scale = 0.15;
    let bird_collider_size = bird_size * bird_scale;

    commands.spawn((
        MovableEntity {
            sprite_renderer: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(-100.0, 0.0, 2.0),
                    scale: Vec3::splat(bird_scale),
                    ..default()
                },
                texture: bird_texture_handle.clone(),
                sprite: Sprite::default(),
                ..default()
            },
            collider: Collider::rectangle(bird_collider_size.x, bird_collider_size.y),
            velocity: LinearVelocity::default(),
        },
        Bird,
        RigidBody::Kinematic,
    ));
}

pub fn move_pipes(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &LinearVelocity), With<Pipe>>,
    mut score: ResMut<Score>,
    time: Res<Time>,
) {
    for (entity, mut pipe_transform, pipe_velocity) in query.iter_mut() {
        pipe_transform.translation.x += pipe_velocity.0.x * time.delta_seconds();
        if pipe_transform.translation.x < -800.0 {
            // Despawn the pipe entity
            commands.entity(entity).despawn();
            score.0 += 1;
        }
    }
}
pub fn check_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<Collision>,
    bird_query: Query<Entity, With<Bird>>,
    mut score: ResMut<Score>,
    mut next_state: ResMut<NextState<crate::GameState>>,
) {
    for collision in collision_events.read() {
        if let Ok(bird_entity) = bird_query.get(collision.0.entity1) {
            println!("Bird collided with a pipe!");
            commands.entity(bird_entity).despawn();
            score.0 = 0;
            next_state.set(crate::GameState::GameOver);
        } else if let Ok(bird_entity) = bird_query.get(collision.0.entity2) {
            println!("Bird collided with a pipe!");
            commands.entity(bird_entity).despawn();
            score.0 = 0;
            next_state.set(crate::GameState::GameOver);
        }
    }
}
