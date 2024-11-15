use bevy::{
    input::mouse::MouseMotion, math::Dir3, pbr::{ExtendedMaterial, MaterialExtension, OpaqueRendererMethod}, prelude::*, render::render_resource::{AsBindGroup, ShaderRef}, transform::TransformSystem, window::{WindowMode, WindowResolution}
};
use bevy_dolly::prelude::*;
use character_controller::*;
use game_lib::GamePlugin;
use space_avian3d_lib::prelude::*;
use game_management::GameLayer;
use avian3d::prelude::*;
use space_prefab::prelude::{PrefabBundle, PrefabPlugin};

mod character_controller;
mod game_management;

// The component tag used to parent to a Dolly Rig
#[derive(Component)]
pub struct MainCamera;

fn main() {
    let mut app = App::default();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resizable: true,
            focused: true,
            title: "Artificer".into(),
            resolution: WindowResolution::new(1600., 900.),
            visible: true,
            mode: WindowMode::Windowed,
            ..default()
        }),
        ..default()
    }))
    .add_plugins((PrefabPlugin, Avian3dPlugin))
    .add_plugins(CharacterControllerPlugin)
    .add_plugins(DollyCursorGrab)
    .add_plugins(GamePlugin)
    //.add_plugins(avian3d::PhysicsPlugins::default().with_length_unit(100.0))
    .add_plugins(MaterialPlugin::<
        ExtendedMaterial<StandardMaterial, MyExtension>,
    >::default())
    .add_systems(Startup, setup)
    //.add_systems(Startup, effects_setup.before(setup))
    .add_systems(Update, Dolly::<MainCamera>::update_active)
    .add_systems(
        PostUpdate,
        update_camera
            .after(PhysicsSet::Sync)
            .before(TransformSystem::TransformPropagate),
    )
    .run();
}

fn setup(
    mut commands: Commands,
    _assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    //mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) {
    // prefab loaded by adding PrefabLoader component to any entity (it will be parent of prefab) or with prefab bundle
    commands
        .spawn(PrefabBundle::new("scenes/lofi_test.scn.ron"))
        .insert(Name::new("Scene_Prefab"));

    // Player
    commands.spawn((
        CharacterControllerBundle::new(Collider::capsule(1.0, 0.4)).with_movement(
            100.0,
            0.92,
            8.0,
            (70f32).to_radians(),
        ),
        CollisionLayers::new(
            GameLayer::Player,
            [GameLayer::Enemy, GameLayer::Ground, GameLayer::Default],
        ),
        Grounded::default(),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        GravityScale(2.0),
    ));

    // Camera
    commands.spawn((
        MainCamera,
        Rig::builder()
            .with(bevy_dolly::prelude::Position::new(Vec3::ZERO))
            .with(YawPitch::new().yaw_degrees(0.0).pitch_degrees(-30.0))
            .with(Smooth::new_position(0.3))
            .with(Smooth::new_rotation(0.3))
            .build(),
        Camera3dBundle {
            transform: Transform::from_xyz(0., 0., 0.).looking_at(Vec3::Z, Vec3::Y),
            ..Default::default()
        },
        RayCaster::new(Vec3::ZERO, Dir3::X),
    ));
}

fn update_camera(
    q0: Query<&Transform, With<CharacterController>>,
    mut q1: Query<&mut Rig>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
    grab_config: Res<DollyCursorGrabConfig>,
    mut query: Query<(&mut RayCaster, &Transform), With<MainCamera>>,
) {
    let player = q0.single().to_owned();
    let mut rig = q1.single_mut();
    let speed: f32 = 20.;

    rig.driver_mut::<bevy_dolly::prelude::Position>().position =
        player.translation + Vec3::new(0., 1., 0.);

    if grab_config.visible {
        return;
    }

    for ev in motion_evr.read() {
        rig.driver_mut::<YawPitch>().rotate_yaw_pitch(
            -ev.delta.x * time.delta_seconds() * speed,
            -ev.delta.y * time.delta_seconds() * speed,
        );
    }

    let (mut caster, cam) = query.single_mut();
    caster.origin = cam.translation;
    caster.direction = cam.forward();
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct MyExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[uniform(100)]
    pub quantize_steps: u32,
}

impl MaterialExtension for MyExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/toon_shader.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/toon_shader.wgsl".into()
    }
}
