use bevy::audio::{PlaybackMode, Volume};
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_xpbd_2d::prelude::*;

mod game;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default().build())
        .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(ShapePlugin)
        .insert_state(GameState::MainMenu)
        .add_event::<game::PlayAudio>()
        .add_systems(Update, audio_events)
        .add_systems(
            OnEnter(GameState::MainMenu),
            spawn_camera.run_if(run_once()),
        )
        .add_systems(OnEnter(GameState::MainMenu), main_menu_setup)
        .add_systems(
            Update,
            main_menu_check_input.run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(OnExit(GameState::MainMenu), clear_entities)
        .add_systems(
            OnEnter(GameState::Playing),
            (game::spawn_clouds, game::spawn_buildings, game::game_setup),
        )
        .add_systems(
            Update,
            (
                game::spawn_pipes,
                game::animate_clouds,
                game::animate_buildings,
                game::bird_controller,
                game::move_pipes,
                game::check_collisions,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), clear_entities)
        .add_systems(OnEnter(GameState::GameOver), game_over_setup)
        .add_systems(
            Update,
            game_over_check_input.run_if(in_state(GameState::GameOver)),
        )
        .add_systems(OnExit(GameState::GameOver), clear_entities)
        /*
        .add_systems(Update, main_menu)
        .add_systems(Update, game_over)*/
        .insert_resource(game::Score(0))
        .insert_resource(game::LastPipeSpawnTime(0.0))
        .insert_resource(game::Gravity(0.0))
        .run();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    MainMenu,
    Playing,
    GameOver,
}

fn audio_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut evt_audio: EventReader<game::PlayAudio>,
) {
    for event in evt_audio.read() {
        //let audio = asset_server.load(event.0.clone());
        commands.spawn(AudioBundle {
            source: asset_server.load(event.0.clone()),
            settings: PlaybackSettings {
                volume: Volume::new(0.05),
                mode: PlaybackMode::Despawn,
                ..default()
            },
        });
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
fn main_menu_setup(mut commands: Commands) {
    commands.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Press SPACE to start".to_string(),
                style: TextStyle {
                    font_size: 50.0,
                    color: Color::BLACK,
                    ..default()
                },
            }],
            ..default()
        },
        ..default()
    });
}

fn main_menu_check_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Transition to Playing state
        next_state.set(GameState::Playing)
    }
}

fn clear_entities(
    mut commands: Commands,
    query: Query<Entity, (Without<Camera>, Without<Window>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn game_over_setup(mut commands: Commands) {
    // Display game over text
    commands.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Game Over\nPress SPACE to restart".to_string(),
                style: TextStyle {
                    font_size: 50.0,
                    color: Color::BLACK,
                    ..default()
                },
            }],
            ..default()
        },
        ..default()
    });
}

fn game_over_check_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Transition to MainMenu state
        next_state.set(GameState::Playing)
    }
}
