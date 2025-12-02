use bevy::prelude::*;

// Точка входа
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)      // окно, рендер, ввод и т.п. :contentReference[oaicite:2]{index=2}
        .add_systems(Startup, setup)      // один раз при старте
        .run();
}

// Запускается один раз при старте приложения
fn setup(mut commands: Commands) {
    // Камера для 2D сцены
    commands.spawn(Camera2d); // 0.17-стиль: достаточно компонента Camera2d :contentReference[oaicite:3]{index=3}

    // Простой цветной квадрат в центре
    commands.spawn((
        Sprite::from_color(              // спрайт из одного цвета :contentReference[oaicite:4]{index=4}
            Color::srgb(0.2, 0.7, 1.0),  // голубовато-синий цвет
            Vec2::new(150.0, 150.0),     // ширина/высота квадрата
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
