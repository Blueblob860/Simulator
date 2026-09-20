use rapier3d::prelude::*;

pub mod device;
pub mod dynamic_collider;

pub struct PhysicalState {
    gravity: Vec3 = Vector::new(0.0, -9.81, 0.0),
    int_params: IntegrationParameters,
    phys_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    imp_jointset: ImpulseJointSet,
    multi_jointset: MultibodyJointSet,
    ccd_solver: CCDSolver,
    rb_set: RigidBodySet,
    collider_set: ColliderSet
}

impl PhysicalState {
    pub fn new() -> Self {
        let mut int_params = IntegrationParameters::default();
        int_params.set_inv_dt(100.0);
        int_params.num_solver_iterations = 12;

        let phys_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = DefaultBroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let imp_jointset = ImpulseJointSet::new();
        let multi_jointset = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let rb_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        let floor_collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0)
            .friction(0.5).restitution(0.05).build();
        collider_set.insert(floor_collider);

        Self {
            int_params,
            phys_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            imp_jointset,
            multi_jointset,
            ccd_solver,
            rb_set,
            collider_set,
            ..
        }
    }

    pub fn step(&mut self) {
        self.phys_pipeline.step(
            self.gravity, 
            &self.int_params, 
            &mut self.island_manager, 
            &mut self.broad_phase, 
            &mut self.narrow_phase, 
            &mut self.rb_set, 
            &mut self.collider_set, 
            &mut self.imp_jointset, 
            &mut self.multi_jointset, 
            &mut self.ccd_solver, 
            &(), 
            &()
        );
    }
}