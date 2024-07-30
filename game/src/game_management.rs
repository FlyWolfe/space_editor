use space_bevy_xpbd_plugin::prelude::bevy_xpbd_3d::prelude::PhysicsLayer;

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Default,
    Player,
    Enemy,
    Ground,
    Projectile,
}