use rapier3d::{dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, ColliderHandle, ColliderSet}};

use crate::app::vertex::Vertex;

struct PhysicsBody {
    rigidbody_handle: RigidBodyHandle,
    colliders: Vec<ColliderHandle>,
}

struct PhysicsBodyBuilder {
    rigidbody: RigidBodyBuilder,
    colliders: Vec<ColliderBuilder>
}

impl PhysicsBodyBuilder {
    pub fn add(&self) {
        
    }

    pub fn build(&self, rbset: &mut RigidBodySet, colset: &mut ColliderSet) -> PhysicsBody {
        let rigidbody = self.rigidbody.clone().build();
        let rigidbody_handle = rbset.insert(rigidbody);
        let colliders = self.colliders.iter().map(|cb| {
            return colset.insert_with_parent(
                cb.clone().build(), 
                rigidbody_handle.clone(), 
                rbset
            );
        }).collect();
        PhysicsBody { rigidbody_handle, colliders }
    }

    //pub fn create_model(&self) -> (Vec<Vertex>, Vec<u32>) {
    //    
    //}
}