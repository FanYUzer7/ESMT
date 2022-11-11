use glium::{glutin, implement_vertex, Surface};
use glium::glutin::event::{WindowEvent};
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::glutin::event::Event;
use MerkleRTree::shape::Rect;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

struct MerkleTreeDrawer {

}

implement_vertex!(Vertex, position);

fn main() {

    let mut events_loop = EventLoop::new();
    let window = WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let vertex1 = Vertex { position: [-0.5, -0.5] };
    let vertex2 = Vertex { position: [ -0.5,  0.5] };
    let vertex3 = Vertex { position: [ 0.5, 0.5] };
    let vertex4 = Vertex { position: [ 0.5, -0.5] };
    let shape1 = vec![vertex1, vertex2, vertex3, vertex4];

    let vertex1 = Vertex { position: [-0.3, -0.3] };
    let vertex2 = Vertex { position: [ -0.3,  0.3] };
    let vertex3 = Vertex { position: [ 0.3, 0.3] };
    let vertex4 = Vertex { position: [ 0.3, -0.3] };
    let shape2 = vec![vertex1, vertex2, vertex3, vertex4];


    let vertex_buffer1 = glium::VertexBuffer::new(&display, &shape1).unwrap();
    let vertex_buffer2 = glium::VertexBuffer::new(&display, &shape2).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::LineLoop);

    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        out vec4 color;
        void main() {
            color = vec4(0.0, 0.0, 0.0, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();


    events_loop.run(move |ev, _, control_flow| {

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);

        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            _ => (),
        }

        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 1.0);
        target.draw(&vertex_buffer1, &indices, &program, &glium::uniforms::EmptyUniforms,
                    &Default::default()).unwrap();
        target.draw(&vertex_buffer2, &indices, &program, &glium::uniforms::EmptyUniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();
    });
}