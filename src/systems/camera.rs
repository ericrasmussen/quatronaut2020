use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
};

use crate::components::{perspective::Perspective, tags::CameraTag};

// use log::info;

// big TODO: as this system gets more complicated, at some point it'll probably
// be worth using ncollide's broad phase collision, which would let us consolidate
// this and collision.rs.
#[derive(SystemDesc)]
pub struct CameraShakeSystem;

impl<'s> System<'s> for CameraShakeSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, CameraTag>,
        Write<'s, Perspective>,
        Read<'s, Time>,
    );

    // hm, why is this running... something being inserted by default
    // maybe the default should be completed?
    fn run(&mut self, (mut transforms, cameras, mut perspective, time): Self::SystemData) {
        for (transform, _camera) in (&mut transforms, &cameras).join() {
            // this uses prepend to keep shaking if we're not done shaking yet,
            // otherwise resets the z axis to 0 (unrotated)
            match perspective.next_z_rotation(time.delta_seconds()) {
                Some(next_z) => transform.prepend_rotation_z_axis(next_z),
                None => transform.set_rotation_z_axis(0.0),
            };

            // we also continue updating the scale as long as the `Perspective` provides
            // Some(next_scale)
            let current_scale = transform.scale().x;
            if let Some(next_scale) = perspective.next_scale(current_scale, time.delta_seconds()) {
                transform.set_scale(next_scale);
            }
        }
    }
}
