use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use crate::EditorState;

pub mod collider;

use crate::prelude::EditorRegistryExt;

pub type Vector = bevy_xpbd_3d::math::Vector;
pub type Scalar = bevy_xpbd_3d::math::Scalar;


pub struct BevyXpbdPlugin;

impl Plugin for BevyXpbdPlugin {
    fn build(&self, app: &mut App) {

        // if !app.is_plugin_added::<bevy_xpbd_3d::prelude::D() {
        
        // }
        app.add_plugins(PhysicsPlugins::default());

        app.editor_registry::<collider::ColliderPrefab>();
        app.editor_registry::<RigidBodyPrefab>();

        app.add_systems(Update, collider::update_collider);
        app.add_systems(Update, rigidbody_type_change_in_editor.run_if(in_state(EditorState::Editor)));
        app.add_systems(Update, rigidbody_type_change.run_if(in_state(EditorState::Game)));
        app.add_systems(OnEnter(EditorState::Editor), force_rigidbody_type_change_in_editor);
        app.add_systems(OnEnter(EditorState::Game), force_rigidbody_type_change);
        app.add_systems(Update, 
            editor_pos_change
                .after(crate::editor::inspector::inspect)
                .run_if(in_state(EditorState::Editor)));
    }
}

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub enum RigidBodyPrefab {
    Dynamic,
    #[default]
    Static,
    Kinematic
}

impl RigidBodyPrefab {
    pub fn to_rigidbody(&self) -> RigidBody {
        match self {
            RigidBodyPrefab::Dynamic => RigidBody::Dynamic,
            RigidBodyPrefab::Static => RigidBody::Static,
            RigidBodyPrefab::Kinematic => RigidBody::Kinematic,
        }
    }

    pub fn to_rigidbody_editor(&self) -> RigidBody {
        RigidBody::Static
    }
}


fn force_rigidbody_type_change_in_editor(
    mut commands : Commands,
    query : Query<(Entity, &RigidBodyPrefab, Option<&Transform>)>
) {
    for (e, tp, transform) in query.iter() {
        commands.entity(e).insert(tp.to_rigidbody_editor());
        if let Some(tr) = transform {
            commands.entity(e).insert(Position(tr.translation));
            commands.entity(e).insert(Rotation(tr.rotation));
        }
    }
}


fn rigidbody_type_change_in_editor(
    mut commands : Commands,
    query : Query<(Entity, &RigidBodyPrefab, Option<&Transform>), Changed<RigidBodyPrefab>>
) {
    for (e, tp , transform) in query.iter() {
        info!("Rigidbody type changed in {:?}", e);
        commands.entity(e).remove::<RigidBody>().insert(tp.to_rigidbody_editor());
        if let Some(tr) = transform {
            commands.entity(e).insert(Position(tr.translation));
            commands.entity(e).insert(Rotation(tr.rotation));
        }
    }
}

fn force_rigidbody_type_change(
    mut commands : Commands,
    query : Query<(Entity, &RigidBodyPrefab)>
) {
    for (e, tp) in query.iter() {
        commands.entity(e).remove::<RigidBody>().insert(tp.to_rigidbody());
    }
}

fn rigidbody_type_change(
    mut commands : Commands,
    query : Query<(Entity, &RigidBodyPrefab), Changed<RigidBodyPrefab>>
) {
    for (e, tp) in query.iter() {
        commands.entity(e).insert(tp.to_rigidbody());
    }
}


pub fn editor_pos_change(
    mut commands : Commands,
    mut query : Query<(&mut Position, &mut Rotation, &Transform), Changed<Transform>>
) {
    for (mut pos, mut rot, transform) in query.iter_mut() {
        // let transform = transform.compute_transform();
        pos.0 = transform.translation;
        rot.0 = transform.rotation;
    }
}