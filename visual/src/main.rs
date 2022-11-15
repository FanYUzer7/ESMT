use glium::{Display, Frame, glutin, implement_vertex, Program, Surface};
use glium::glutin::event::{ElementState, WindowEvent};
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::glutin::event::Event;
use rand::{Rng, thread_rng};
use MerkleRTree::shape::Rect;
use types::hash_value::HashValue;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    lcolor: [f32; 3],
}

const COLOR_SET: [[f32;3]; 7]= [
    [0.0, 0.0, 0.0], // hei
    [0.0, 0.0, 1.0], // lan
    [0.0, 1.0, 0.0], // lv
    [0.0, 1.0, 1.0], // qing
    [1.0, 0.0, 0.0], // hong
    [1.0, 0.0, 1.0], // zi
    [1.0, 1.0, 0.0], // huang
];

struct MerkleTreeDrawer {
    pub frame: Frame,
}

impl MerkleTreeDrawer {
    pub fn new(frame: Frame) -> Self {
        Self {
            frame
        }
    }

    pub fn clear_color(&mut self) {
        self.frame.clear_color(1.0, 1.0, 1.0, 0.0);
    }

    pub fn draw(&mut self, source: &(u32, Rect<f32, 2>), display: &Display, program: &Program) {
        // 0--2
        // | /|
        // |/ |
        // 1--3
        let (height, rect) = source;
        let color = COLOR_SET[(*height) as usize].clone();
        let vertex0 = Vertex { position: [rect._min[0], rect._max[1]], lcolor: color.clone()};
        let vertex1 = Vertex { position: [rect._min[0], rect._min[1]], lcolor: color.clone()};
        let vertex2 = Vertex { position: [rect._max[0], rect._max[1]], lcolor: color.clone()};
        let vertex3 = Vertex { position: [rect._max[0], rect._min[1]], lcolor: color.clone()};
        let shape = vec![vertex0, vertex1, vertex3, vertex2];
        let buffer = glium::VertexBuffer::new(display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::LineLoop);
        self.frame.draw(&buffer, &indices, program, &glium::uniforms::EmptyUniforms,
                        &Default::default()).unwrap();
    }

    pub fn draw_node(&mut self, rect: &Rect<f32, 2>, display: &Display, program: &Program) {
        let color = [0.64, 0.16, 0.16];
        let vertex0 = Vertex { position: [rect._min[0]-0.01, rect._max[1]+0.01], lcolor: color.clone()};
        let vertex1 = Vertex { position: [rect._min[0]-0.01, rect._min[1]-0.01], lcolor: color.clone()};
        let vertex2 = Vertex { position: [rect._max[0]+0.01, rect._max[1]+0.01], lcolor: color.clone()};
        let vertex3 = Vertex { position: [rect._max[0]+0.01, rect._min[1]-0.01], lcolor: color.clone()};
        let shape = vec![vertex0, vertex1, vertex2, vertex3];
        let buffer = glium::VertexBuffer::new(display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
        self.frame.draw(&buffer, &indices, program, &glium::uniforms::EmptyUniforms,
                        &Default::default()).unwrap();
    }

    pub fn finish(self) {
        self.frame.finish().unwrap();
    }
}

implement_vertex!(Vertex, position, lcolor);

fn main() {

    let events_loop = EventLoop::new();
    let window = WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        in vec3 lcolor;
        out vec3 attr;
        void main() {
            attr = lcolor;
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        in vec3 attr;
        out vec4 color;
        void main() {
            color = vec4(attr, 0.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src,
                                              None).unwrap();

    let mut tree = MerkleRTree::node::MerkleRTree::<f32, 2, 3>::new();
    let mut rng = thread_rng();
    let mut nodes = vec![];
    let mut objs = vec![];
    let mut inserted = false;

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
                WindowEvent::KeyboardInput { input, ..} => {
                    // println!("get keyboard input: {:?}", input);
                    if input.scancode == 23 && input.state == ElementState::Released {
                        let x = rng.gen_range(-1.0f32..1.0f32);
                        let y = rng.gen_range(-1.0f32..1.0f32);
                        println!("insert object: {:?}", [x, y]);
                        tree.insert("test".to_string(), [x,y], HashValue::zero());
                        (nodes, objs) = tree.display();
                        inserted = true;
                    } else if input.scancode == 19 && input.state == ElementState::Released {
                        tree = MerkleRTree::node::MerkleRTree::<f32, 2, 3>::new();
                    }
                }
                _ => return,
            },
            _ => (),
        }

        let mut tree_drawer = MerkleTreeDrawer::new(display.draw());
        tree_drawer.clear_color();
        if inserted {
            for (idx,node) in nodes.iter().enumerate() {
                println!("[{}] tree node height: {}, area: {}", idx, node.0, node.1);
                tree_drawer.draw(node, &display, &program);
            }
            inserted = false;
        } else {
            for node in nodes.iter() {
                tree_drawer.draw(node, &display, &program);
            }
        }
        for obj in objs.iter() {
            tree_drawer.draw_node(obj, &display, &program);
        }
        tree_drawer.finish();
    });
}