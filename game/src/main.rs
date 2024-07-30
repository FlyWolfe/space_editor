use bevy::{
    input::mouse::MouseMotion,
    pbr::{ExtendedMaterial, MaterialExtension, OpaqueRendererMethod},
    prelude::*,
    reflect::TypePath,
    render::{color, render_resource::{AsBindGroup, ShaderRef}},
    transform::TransformSystem,
    window::{WindowMode, WindowResolution},
};
use bevy_dolly::prelude::*;
use character_controller::*;
use game_lib::GamePlugin;
use game_management::GameLayer;
use shape::Capsule;
use space_bevy_xpbd_plugin::prelude::bevy_xpbd_3d::prelude::*;
use space_bevy_xpbd_plugin::XpbdPlugin;
use space_prefab::prelude::{PrefabBundle, PrefabPlugin};

mod character_controller;
mod game_management;
mod projectile;

// The component tag used to parent to a Dolly Rig
#[derive(Component)]
pub struct MainCamera;

fn main() {
    let mut app = App::new();
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
    .add_plugins((
        PrefabPlugin,
        CharacterControllerPlugin,
        DollyCursorGrab,
        GamePlugin,
        XpbdPlugin,
    ))
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
    .add_systems(Update, projectile::mouse_input)
    .add_systems(Update, projectile::update_projectiles)
    .run();
}

fn setup(
    mut commands: Commands,
    _assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) {
    // prefab loaded by adding PrefabLoader component to any entity (it will be parent of prefab) or with prefab bundle
    commands
        .spawn(PrefabBundle::new("scenes/play_scene.scn.ron"))
        .insert(Name::new("Prefab"));

    // Player
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Capsule3d::new(0.5, 1.0)),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            material: materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color: Color::rgb(0.1, 0.1, 0.8),
                    // can be used in forward or deferred mode.
                    opaque_render_method: OpaqueRendererMethod::Auto,
                    // in deferred mode, only the PbrInput can be modified (uvs, color and other material properties),
                    // in forward mode, the output can also be modified after lighting is applied.
                    // see the fragment shader `extended_material.wgsl` for more info.
                    // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
                    // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
                    ..Default::default()
                },
                extension: MyExtension { quantize_steps: 3 },
            }),
            ..default()
        },
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
            .with(Arm::new((Vec3::Z * 10.0) + (Vec3::Y * 1.0)))
            .build(),
        Camera3dBundle {
            transform: Transform::from_xyz(0., 1., 5.).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        RayCaster::new(Vec3::ZERO, Direction3d::X),
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
