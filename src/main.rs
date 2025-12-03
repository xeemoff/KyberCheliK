use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::{SpriteBundle, TextureAtlas, TextureAtlasLayout};
use bevy_xpbd_2d::prelude::*;

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
const TILE_SIZE: f32 = 48.0;
const PLAYER_SIZE: Vec2 = Vec2::new(32.0, 48.0);
const PLAYER_SPAWN: Vec2 = Vec2::new(-400.0, 200.0);
const DASH_DURATION: f32 = 0.18;
const DASH_COOLDOWN: f32 = 0.35;
const BACKGROUND_COLOR: Color = Color::srgb(0.08, 0.09, 0.12);

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Gravity(Vec2::NEG_Y * 1500.0))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "KyberCheliK Platformer".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(LevelPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// --- Level -----------------------------------------------------------------

struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_level);
    }
}

#[derive(Component)]
struct LevelTile;

const LEVEL_MAP: [&str; 11] = [
    "####################",
    "#..................#",
    "#.................##",
    "#..................#",
    "#...............#..#",
    "#...###........##..#",
    "#..................#",
    "#.........###......#",
    "#..................#",
    "#..................#",
    "####################",
];

fn setup_level(mut commands: Commands) {
    let origin = Vec2::new(-TILE_SIZE * LEVEL_MAP[0].len() as f32 * 0.5, -160.0);

    for (row, line) in LEVEL_MAP.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch != '#' {
                continue;
            }

            let position = origin
                + Vec2::new(
                    col as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                    -(row as f32) * TILE_SIZE,
                );

            commands.spawn((
                LevelTile,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.20, 0.22, 0.25),
                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(position.x, position.y, 0.0),
                    ..default()
                },
                RigidBody::Static,
                Collider::rectangle(TILE_SIZE, TILE_SIZE),
            ));
        }
    }
}

// --- Player ----------------------------------------------------------------

struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerConfig>()
            .add_systems(Startup, (setup_player_assets, spawn_player))
            .add_systems(
                Update,
                (
                    player_input,
                    update_player_state,
                    animate_player,
                    apply_ground_snap,
                )
                    .chain(),
            );
    }
}

#[derive(Resource)]
struct PlayerConfig {
    move_speed: f32,
    jump_speed: f32,
    dash_speed: f32,
    air_control: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            move_speed: 360.0,
            jump_speed: 640.0,
            dash_speed: 820.0,
            air_control: 0.6,
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component, Copy, Clone, Eq, PartialEq, Debug, Default)]
enum PlayerState {
    #[default]
    Standing,
    Jumping,
    Falling,
    Dashing,
}

#[derive(Component, Debug)]
struct Facing(f32);

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct DashTimers {
    duration: Timer,
    cooldown: Timer,
}

#[derive(Component)]
struct PlayerAnimation;

#[derive(Component)]
struct Grounded(bool);

#[derive(Resource, Clone)]
struct PlayerAssets {
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

fn setup_player_assets(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let pixels: Vec<[u8; 4]> = vec![
        [255, 255, 255, 255], // idle
        [120, 180, 255, 255], // jump
        [255, 200, 120, 255], // fall
        [255, 120, 160, 255], // dash
    ];

    let mut data = Vec::new();
    for rgba in &pixels {
        data.extend_from_slice(rgba);
    }

    let image = Image::new_fill(
        Extent3d {
            width: pixels.len() as u32,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    let texture = images.add(image);
    let layout = atlases.add(TextureAtlasLayout::from_grid(
        UVec2::ONE,
        pixels.len() as u32,
        1,
        None,
        None,
    ));

    commands.insert_resource(PlayerAssets { texture, layout });
}

fn spawn_player(mut commands: Commands, assets: Res<PlayerAssets>) {
    commands.spawn((
        SpriteBundle {
            texture: assets.texture.clone(),
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
            transform: Transform::from_xyz(PLAYER_SPAWN.x, PLAYER_SPAWN.y, 1.0),
            ..default()
        },
        TextureAtlas {
            layout: assets.layout.clone(),
            index: 0,
        },
        Player,
        PlayerState::Standing,
        Facing(1.0),
        PlayerAnimation,
        Grounded(false),
        AnimationTimer(Timer::from_seconds(0.14, TimerMode::Repeating)),
        DashTimers {
            duration: Timer::from_seconds(DASH_DURATION, TimerMode::Once),
            cooldown: Timer::from_seconds(DASH_COOLDOWN, TimerMode::Once),
        },
        RigidBody::Dynamic,
        Collider::rectangle(PLAYER_SIZE.x, PLAYER_SIZE.y),
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity(Vec2::ZERO),
        Friction::new(1.0),
        Restitution::new(0.0),
    ));
}

fn player_input(
    time: Res<Time>,
    config: Res<PlayerConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Res<Gamepads>,
    button_input: Res<ButtonInput<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut query: Query<
        (
            &mut LinearVelocity,
            &mut PlayerState,
            &mut Facing,
            &mut DashTimers,
            &Grounded,
        ),
        With<Player>,
    >,
) {
    let (mut velocity, mut state, mut facing, mut dash_timers, grounded) = query.single_mut();

    let mut axis = 0.0;
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        axis -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        axis += 1.0;
    }

    for gamepad in gamepads.iter() {
        axis += axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
            .unwrap_or(0.0);
    }

    let on_ground = grounded.0;
    let desired_speed = if on_ground {
        config.move_speed
    } else {
        config.move_speed * config.air_control
    };

    velocity.x = axis * desired_speed;

    if axis.abs() > 0.1 {
        facing.0 = axis.signum();
    }

    dash_timers.cooldown.tick(time.delta());

    let jump_pressed = keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyW)
        || gamepads
            .iter()
            .any(|g| button_input.just_pressed(GamepadButton::new(g, GamepadButtonType::South)));

    if on_ground && jump_pressed {
        velocity.y = config.jump_speed;
        *state = PlayerState::Jumping;
    }

    let dash_pressed = keyboard.just_pressed(KeyCode::ShiftLeft)
        || keyboard.just_pressed(KeyCode::ShiftRight)
        || gamepads
            .iter()
            .any(|g| button_input.just_pressed(GamepadButton::new(g, GamepadButtonType::East)));

    if dash_pressed && dash_timers.cooldown.finished() {
        dash_timers.duration.reset();
        dash_timers.cooldown.reset();
        *state = PlayerState::Dashing;
        velocity.y = 0.0;
        velocity.x = facing.0 * config.dash_speed;
    }

    if matches!(*state, PlayerState::Dashing) {
        if dash_timers.duration.tick(time.delta()).finished() {
            *state = PlayerState::Falling;
        } else {
            velocity.y = 0.0;
            velocity.x = facing.0 * config.dash_speed;
        }
    }
}

fn update_player_state(
    mut query: Query<
        (
            &LinearVelocity,
            &mut PlayerState,
            &mut Grounded,
            &CollidingEntities,
            &GlobalTransform,
        ),
        With<Player>,
    >,
    level_transforms: Query<&GlobalTransform, With<LevelTile>>,
) {
    let (velocity, mut state, mut grounded, collisions, transform) = query.single_mut();
    let position = transform.translation().truncate();

    grounded.0 = is_grounded(position, collisions, &level_transforms);

    match *state {
        PlayerState::Standing => {
            if !grounded.0 {
                *state = PlayerState::Falling;
            }
        }
        PlayerState::Jumping => {
            if velocity.y <= 0.0 {
                *state = PlayerState::Falling;
            }
        }
        PlayerState::Falling => {
            if grounded.0 {
                *state = PlayerState::Standing;
            }
        }
        PlayerState::Dashing => {
            // handled in input system
        }
    }
}

fn is_grounded(
    player_pos: Vec2,
    collisions: &CollidingEntities,
    transforms: &Query<&GlobalTransform, With<LevelTile>>,
) -> bool {
    collisions.iter().any(|entity| {
        if let Ok(transform) = transforms.get(*entity) {
            return transform.translation().y < player_pos.y - PLAYER_SIZE.y * 0.45;
        }
        false
    })
}

fn apply_ground_snap(mut query: Query<(&mut Transform, &Grounded), With<Player>>) {
    // Helps keep the player sitting on the floor instead of hovering because of numerical errors.
    if let Ok((mut transform, grounded)) = query.get_single_mut() {
        if grounded.0 {
            transform.translation.y = transform.translation.y.round();
        }
    }
}

fn animate_player(
    time: Res<Time>,
    mut query: Query<
        (
            &PlayerState,
            &mut TextureAtlas,
            &mut AnimationTimer,
            &mut Sprite,
        ),
        With<PlayerAnimation>,
    >,
) {
    let (state, mut atlas, mut timer, mut sprite) = query.single_mut();

    let frame_range = match state {
        PlayerState::Standing => 0..=0,
        PlayerState::Jumping => 1..=1,
        PlayerState::Falling => 2..=2,
        PlayerState::Dashing => 2..=3,
    };

    if frame_range.start() == frame_range.end() {
        atlas.index = *frame_range.start();
        sprite.color = Color::WHITE;
        return;
    }

    sprite.color = Color::srgb(1.0, 0.8, 0.8);

    timer.tick(time.delta());
    if timer.just_finished() {
        atlas.index += 1;
        if atlas.index < *frame_range.start() || atlas.index > *frame_range.end() {
            atlas.index = *frame_range.start();
        }
    }
}
