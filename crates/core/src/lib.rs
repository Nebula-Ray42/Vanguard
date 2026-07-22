use rapier3d::prelude::*;

use render_api::{RenderCommand, RenderCommandList, Vec3};

pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub gravity: Vector<Real>,
    pub box_handle: RigidBodyHandle,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        // ==========================================
        // フェーズ1：ローカル変数の準備（仕込み）
        // ==========================================

        // let mut 変数名 = 初期値; で宣言します（: ではなく = です）
        // 後で insert して状態が変わるため mut（可変）にします
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        // 箱を生成してセットに登録する
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 10.0, 0.0])
            .build();
        let box_handle = rigid_body_set.insert(rigid_body);

        let box_collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5).build();
        collider_set.insert_with_parent(box_collider, box_handle, &mut rigid_body_set);

        // ==========================================
        // 床の生成（固定された巨大な板）
        // ==========================================
        let floor_collider = ColliderBuilder::cuboid(50.0, 0.1, 50.0).build();
        collider_set.insert(floor_collider);

        // ==========================================
        // フェーズ2：構造体の実体化（組み立てて返す）
        // ==========================================
        Self {
            // （本来は rigid_body_set: rigid_body_set, ですが、変数名とフィールド名が同じ場合は省略できます）
            rigid_body_set,
            collider_set,
            box_handle,

            // その他の設定項目を直接埋め込む
            integration_parameters: IntegrationParameters {
                dt: 1.0 / 60.0, // 60FPS固定
                ..Default::default()
            },
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            gravity: vector![0.0, -9.81, 0.0],
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }
}

// （PhysicsWorld の定義などはそのまま）

pub struct GameState {
    pub frame_count: u64,
    pub physics: PhysicsWorld, // 本物の物理世界を保持
}

impl GameState {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            physics: PhysicsWorld::new(),
        }
    }

    /// 1フレーム（1/60秒）ごとに呼ばれる純粋なロジック更新処理
    pub fn tick(&mut self) -> RenderCommandList {
        self.frame_count += 1;

        // 1. 物理シミュレーションを1ステップ進める
        self.physics.step();

        // 2. 箱の現在の座標を取得する
        let box_body = self
            .physics
            .rigid_body_set
            .get(self.physics.box_handle)
            .unwrap();
        let pos = box_body.translation();

        // 3. 描画コマンドとして出力する
        let mut cmd_list = RenderCommandList::new();
        cmd_list.commands.push(RenderCommand {
            mesh_id: 1, // 箱のメッシュIDを仮に1とする
            position: Vec3 {
                x: pos.x,
                y: pos.y,
                z: pos.z,
            },
        });

        cmd_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_fall() {
        let mut state = GameState::new();

        // 初期位置は Y=10.0
        assert!((state.tick().commands[0].position.y - 10.0).abs() < 0.1);

        // 充分なフレーム（約3秒分）進める
        for _ in 0..180 {
            state.tick();
        }

        // 最終的に床(Y=0.0)付近で静止していることを確認
        // 箱のサイズが0.5(半径)なので、中心座標は Y=0.5 付近になるはず
        let final_y = state.tick().commands[0].position.y;
        println!("Final Box Y Position: {}", final_y);
        assert!(final_y >= 0.49 && final_y <= 0.51);
    }
}
