use bevy::{
    prelude::*,
    window::{WindowMode, WindowResolution},
};
use game_lib::GamePlugin;
use space_bevy_xpbd_plugin::XpbdPlugin;
use space_prefab::prelude::{PrefabBundle, PrefabPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resizable: true,
            focused: true,
            title: "Your Game".into(),
            resolution: WindowResolution::new(1600., 900.),
            visible: true,
            mode: WindowMode::Windowed,
            ..default()
        }),
        ..default()
    }))
    .add_plugins((PrefabPlugin, GamePlugin, XpbdPlugin))
    .add_systems(Startup, setup)
    .run();
}

fn setup(mut commands: Commands, _assets: Res<AssetServer>) {
    // prefab loaded by adding PrefabLoader component to any entity (it will be parent of prefab) or with prefab bundle
    commands
        .spawn(PrefabBundle::new("scenes/play_scene.scn.ron"))
        .insert(Name::new("Prefab"));

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
