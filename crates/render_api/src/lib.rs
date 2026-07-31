use flatbuffers::FlatBufferBuilder;

#[allow(unused_imports, dead_code, clippy::all)]
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/render_command_generated.rs"));
}

use generated::rey_engine::render::{Color, RenderCommand, RenderCommandArgs};

unsafe extern "C" {
    fn execute_render_command(ptr: *const u8, len: usize);
}

pub fn send_clear_color() {
    let mut builder = FlatBufferBuilder::with_capacity(1024);

    let color = Color::new(0.1, 0.2, 0.3, 1.0);
    let command = RenderCommand::create(&mut builder, &RenderCommandArgs {
        clear_color: Some(&color),
    });

    builder.finish(command, None);

    let buf = builder.finished_data();

    unsafe {
        // C++の関数を直接呼び出す
        execute_render_command(buf.as_ptr(), buf.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatbuffers_ffi() {
        send_clear_color();
    }
}
