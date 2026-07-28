#[cfg(test)]
mod tests {
    use flatbuffers::FlatBufferBuilder;
    use shared::engine_data::{
        Entity, EntityArgs, Scene, SceneArgs, Transform, TransformArgs, Vec3, Vec4,
    };

    fn euler_to_quaternion(pitch: f32, yaw: f32, roll: f32) -> (f32, f32, f32, f32) {
        let (sp, cp) = (pitch * 0.5).sin_cos();
        let (sy, cy) = (yaw * 0.5).sin_cos();
        let (sr, cr) = (roll * 0.5).sin_cos();
        (
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
            cr * cp * cy + sr * sp * sy,
        )
    }

    #[test]
    fn test_flatbuffers_memory_size() {
        let mut builder = FlatBufferBuilder::new();

        let pitch_rad = 0.0_f32;
        let yaw_rad = 0.0_f32;
        let roll_rad = 0.0_f32;

        let (qx, qy, qz, qw) = euler_to_quaternion(pitch_rad, yaw_rad, roll_rad);

        let pos = Vec3::new(1.0, 2.0, 3.0);
        let rot = Vec4::new(qx, qy, qz, qw);
        let scale = Vec3::new(1.0, 1.0, 1.0);

        let transform_offset = Transform::create(
            &mut builder,
            &TransformArgs {
                position: Some(&pos),
                rotation: Some(&rot),
                scale: Some(&scale),
            },
        );

        let entity_offset = Entity::create(
            &mut builder,
            &EntityArgs {
                id: 777,
                mesh_id: 1,
                transform: Some(transform_offset),
            },
        );

        let entities_vec = builder.create_vector(&[entity_offset]);
        let scene_offset = Scene::create(
            &mut builder,
            &SceneArgs {
                entities: Some(entities_vec),
            },
        );

        builder.finish(scene_offset, None);
        let bytes = builder.finished_data();

        println!("========================================");
        println!("シリアライズ完了");
        println!("総バイト数: {} bytes", bytes.len());
        println!("========================================");

        assert!(!bytes.is_empty());
    }
}
