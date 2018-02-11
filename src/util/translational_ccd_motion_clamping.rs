use std::cell::RefCell;
use alga::general::Real;
use nalgebra::{self, Translation3, Isometry3};
use ncollide::utils::data::hash_map::HashMap;
use ncollide::utils::data::hash::UintTWHash;
use ncollide::bounding_volume::BoundingVolume;
use ncollide::query;
use ncollide::bounding_volume;
use ncollide::world::{CollisionGroups, CollisionObjectHandle, CollisionObject3, CollisionWorld};
use world::{CollideWorld, CollisionDataExtra};
use point::Point;

struct CCDCollisionObject {
    body:            CollisionObjectHandle,
    sqthreshold:     f32,
    last_center:     Point,
    accept_zero:     bool
}

impl CCDCollisionObject {
    fn new(body: &CollisionObject3<f32, CollisionDataExtra>, threshold: f32, pos: Isometry3<f32>) -> CCDCollisionObject {
        CCDCollisionObject {
            sqthreshold:     threshold * threshold,
            last_center:     Point::from_coordinates(pos.translation.vector),
            body:            body.handle(),
            accept_zero:     true
        }
    }
}

/// Handles Continuous Collision Detection.
pub struct TranslationalCCDMotionClamping {
    objects: HashMap<usize, CCDCollisionObject, UintTWHash>,
}

impl TranslationalCCDMotionClamping {
    /// Creates a new `TranslationalCCDMotionClamping` to enable continuous collision detection to
    /// fast-moving rigid bodies.
    pub fn new() -> TranslationalCCDMotionClamping {
        TranslationalCCDMotionClamping {
            objects:                   HashMap::new(UintTWHash::new()),
        }
    }

    /// Enables continuous collision for the given rigid body.
    pub fn add_ccd_to(&mut self,
                      body:       &CollisionObject3<f32, CollisionDataExtra>,
                      motion_threshold: f32,
                      pos: Isometry3<f32>) {
        let _ = self.objects.insert(body.handle().0,
                                    CCDCollisionObject::new(body, motion_threshold, pos));
    }

    /// Enables continuous collision for the given rigid body.
    pub fn remove_ccd_from(&mut self, body: &CollisionObjectHandle) {
        let _ = self.objects.remove(&body.0);
    }

    /// Update the time of impacts and apply motion clamping when necessary.
    ///
    /// Returns `false` if no clamping was done. If at least one clamping was performed, the
    /// collision word will be updated by this method once all the clamping have been performed.
    pub fn update(&mut self, cw: &mut CollideWorld) -> bool {
        let mut update_collision_world = false;

        // XXX: we should no do this in a sequential order because CCD between two fast
        // CCD-enabled objects will not work properly (it will be biased toward the first object).
        for co1 in self.objects.elements_mut().iter_mut() {
            let (shape, pos) = {
                let obj1 = match cw.collision_object(co1.value.body) {
                    Some(o) => o,
                    None => continue,
                };
                (obj1.shape().clone(), obj1.position().clone())
            };

            let movement = Point::from_coordinates(pos.translation.vector) - co1.value.last_center;

            if nalgebra::norm_squared(&movement) > co1.value.sqthreshold {
                // Use CCD for this object.
                let obj1_uid = co1.value.body;

                let last_transform = Translation3::from_vector(-movement) * pos;
                let begin_aabb     = bounding_volume::aabb(shape.as_ref(), &last_transform);
                let end_aabb       = bounding_volume::aabb(shape.as_ref(), &pos);
                let swept_aabb     = begin_aabb.merged(&end_aabb);

                /*
                 * Find the minimum TOI.
                 */
                let mut min_toi   = nalgebra::one::<f32>();
                let mut toi_found = false;
                let dir = movement.clone();

                let _eps = ::std::f32::EPSILON;

                // XXX: handle groups.
                let all_groups = CollisionGroups::new();

                // FIXME: performing a convex-cast here would be much more efficient.
                for co2 in cw.interferences_with_aabb(&swept_aabb, &all_groups) {
                    if co2.handle() != obj1_uid {
                        let obj2 = co2;

                        let toi = query::time_of_impact(
                            &last_transform,
                            &dir,
                            shape.as_ref(),
                            &obj2.position(),
                            &nalgebra::zero(), // Assume the other object does not move.
                            obj2.shape().as_ref());

                        match toi {
                            Some(t) => {
                                if t <= min_toi { // we need the equality case to set the `toi_found` flag.
                                    toi_found = true;

                                    if t > _eps || co1.value.accept_zero {
                                        min_toi = t;
                                    }
                                }
                            },
                            None => { }
                        }
                    }
                }

                /*
                 * Revert the object translation at the toi.
                 */
                if toi_found {
                    {
                        let mut obj1 = cw.collision_object_mut(co1.value.body).unwrap();
                        let trans = Translation3::from_vector(-dir * (nalgebra::one::<f32>() - min_toi));
                        obj1.set_position(trans * pos);
                    }
                    co1.value.accept_zero = false;

                    // We moved the object: ensure the broad phase takes that in account.
                    cw.set_position(co1.value.body, pos.clone());
                    update_collision_world = true;
                }
                else {
                    co1.value.accept_zero = true;
                }

                /*
                 FIXME: * Activate then deactivate all the sensors that should have been traversed by the
                 * rigid body (we do not activate those that the rigid body entered without
                 * leaving).
                 */
                // self.intersected_sensors_cache.sort();
                // for sensor in self.intersected_sensors_cache.iter() {
                //     if sensor.0 < min_toi {
                //         let bsensor = sensor.borrow();

                //         // See if at the final rigid body position the sensor is still intersected.
                //         // NOTE: we are assuming the tensor is convex (handling concave cases would
                //         // be to complicated without much uses).
                //         if !query::test_interference(
                //             obj1.position(),
                //             obj1_shape,
                //             bsensor.position(),
                //             bsensor.shape_ref()) {
                //             // FIXME: activate the collision-start and collision-end signals for
                //             // this sensor.
                //         }
                //         // Otherwise do not trigger this sensor just yet. This will be done during
                //         // the next narrow phase update.
                //     }
                // }
            }

            co1.value.last_center = Point::from_coordinates(pos.translation.vector);
        }

        if update_collision_world {
            cw.update();
            true
        }
        else {
            false
        }
    }
}
