#![allow(unused_variables)]
#![allow(dead_code)]

use crate::ea::bsv3::{Frame, Layer, BSV3};
use crate::ea::file_buffer::FileBuffer;
use crate::ea::rgb::{la88_to_rgba8888, rgba4444_to_rgba8888};
use colored::Colorize;
use delaunator::{triangulate, Point};
use std::collections::HashMap;
use tetra::graphics::mesh::{
    BufferUsage, IndexBuffer, Mesh, Vertex, VertexBuffer,
};
use tetra::graphics::text::{Font, Text};
use tetra::graphics::{
    self, BlendState, Canvas, Color, DrawParams, FilterMode, Texture, TextureFormat,
};
use tetra::math::num_traits::abs;
use tetra::math::{Mat4, Vec2};
use tetra::time::Timestep;
use tetra::{input, time, Context, ContextBuilder, Event, State, TetraError};

mod ea;

const WINDOW_WIDTH: f32 = 1024.0 + 256.0;
const WINDOW_HEIGHT: f32 = 1024.0 + 256.0;
const CANVAS_SIZE: f32 = 2048.0;
const CANVAS_HALF: f32 = CANVAS_SIZE / 2.0;
const DEBUG_LAYERS: bool = false;
const GREEN_BG: bool = false;
const SAVE_CANVAS: bool = false;

fn main() -> Result<(), TetraError> {
    if SAVE_CANVAS {
        std::fs::create_dir_all("pngs").unwrap();
    }

    ContextBuilder::new("TSTO BSV3", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .quit_on_escape(true)
        .show_mouse(true)
        .multisampling(16)
        .resizable(true)
        .timestep(Timestep::Fixed(24.0))
        .build()?
        .run(GameState::new)
}

struct GameState {
    scene: Scene,
    clip_canvas: Canvas,
    clip_index: usize,
    clip_id: u16,
    clip_name: String,
    mouse_down: bool,
    font: Font,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let file_name = "./building.bsv3";

        let font =
            Font::from_vector_file_data(ctx, include_bytes!("../UbuntuMono-Regular.ttf"), 18.0)
                .ok()
                .unwrap();

        let scene = Scene::new(ctx, file_name, font.clone())?;

        Ok(GameState {
            scene,
            clip_canvas: Canvas::new(ctx, CANVAS_SIZE as i32, CANVAS_SIZE as i32)?,
            clip_index: 0,
            clip_id: 0,
            clip_name: String::from(""),
            mouse_down: false,
            font,
        })
    }

    fn save_canvas(&mut self, ctx: &mut Context) {
        let tex_data = self.clip_canvas.texture().get_data(ctx);

        let index = self.scene.get_index();
        let png_path = format!("pngs/{}_{:03}.png", self.scene.bsv3.file.name, index);
        if std::path::Path::new(&png_path).exists() {
            return;
        }

        image::save_buffer_with_format(
            png_path,
            &tex_data.as_bytes(),
            CANVAS_SIZE as u32,
            CANVAS_SIZE as u32,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .unwrap();
    }
}

struct Scene {
    bsv3: BSV3,
    texture: Texture,
    timer: usize,
    scale: Vec2<f32>,
    offset_x: f32,
    offset_y: f32,
    animation: usize,
    frames: HashMap<String, (Mesh, Mat4<f32>)>,
    font: Font,
}

impl Scene {
    fn new(ctx: &mut Context, file_path: &str, font: Font) -> Result<Scene, TetraError> {
        if file_path.is_empty() {
            return Ok(Scene {
                bsv3: BSV3::new(String::from("")),
                texture: Texture::from_data(ctx, 0, 0, TextureFormat::R8, &*vec![])?,
                timer: 0,
                scale: Vec2::new(1.0, 1.0),
                offset_x: 0.0,
                offset_y: 400.0,
                animation: 0,
                frames: HashMap::new(),
                font,
            });
        }

        let mut bsv3 = BSV3::new(String::from(file_path));

        bsv3.parse();

        // println!("Sprites: {:#?}", file.sprites);

        let texture_file = format!("{}.rgb", bsv3.file.name);
        let mut texture_path = format!("{}{}", bsv3.file.folder, texture_file);

        if !bsv3.image_name.is_empty() {
            texture_path = format!("{}{}", bsv3.file.folder, bsv3.image_name);
        }

        let start_time = std::time::Instant::now();
        print!("{} {}\n", "Parsing".blue(), texture_file);
        let mut texture_file = FileBuffer::new(texture_path);
        texture_file.skip(3);
        let format = texture_file.read_uint_8();
        println!("Format: {}", format);
        let width = texture_file.read_uint_16();
        println!("Width: {}", width);
        let height = texture_file.read_uint_16();
        println!("Height: {}", height);
        let texture_data = texture_file.read_remaining();
        println!("Texture: {}x{}", width, height);

        let texture_data_rgba8;

        if format == 0x20 {
            texture_data_rgba8 = rgba4444_to_rgba8888(texture_data, false);
        } else if format == 0x60 {
            texture_data_rgba8 = la88_to_rgba8888(texture_data, false);
        } else {
            texture_data_rgba8 = texture_data;
        }

        let mut texture = Texture::from_data(
            ctx,
            width as i32,
            height as i32,
            TextureFormat::Rgba8,
            &*texture_data_rgba8,
        )?;
        texture.set_filter_mode(ctx, FilterMode::Linear);

        let end_time = std::time::Instant::now();
        print!("{} {:?}\n", "Done in".green(), end_time - start_time);

        Ok(Scene {
            bsv3,
            texture,
            timer: 0,
            scale: Vec2::new(1.0, 1.0),
            offset_x: 0.0,
            offset_y: 400.0,
            animation: 0,
            frames: HashMap::new(),
            font,
        })
    }

    pub fn transform_points(
        &self,
        destination_points: Vec<(f32, f32)>,
        matrix: Vec<f64>,
    ) -> Vec<Point> {
        let mut points = vec![];

        for point in destination_points {
            let x = point.0 as f64;
            let y = point.1 as f64;

            // println!("Point: {:?}", point);
            // println!("Matrix: {:?}", matrix);

            let point = Point {
                x: matrix[0] * x + matrix[1] * y + matrix[4],
                y: matrix[2] * x + matrix[3] * y + matrix[5],
            };

            // println!("Point: {:?}", point);
            // println!("-----------------------------------------------");

            points.push(point);
        }

        points
    }

    pub fn canvas_to_mesh(&mut self, ctx: &mut Context, canvas: Canvas) -> Option<Mesh> {
        let source_points: Vec<(f32, f32)> = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

        let width = canvas.width() as f32;
        let height = canvas.height() as f32;

        let destination_points: Vec<(f32, f32)> =
            vec![(0.0, 0.0), (width, 0.0), (width, height), (0.0, height)];
        /*
         * Generate Vertex Buffer
         */
        let mut vertices = vec![];
        for index in 0..destination_points.len() {
            vertices.push(Vertex::new(
                Vec2::new(destination_points[index].0, destination_points[index].1),
                Vec2::new(source_points[index].0, source_points[index].1),
                Color::WHITE,
            ));
        }
        let vertex_buffer = VertexBuffer::with_usage(ctx, &*vertices, BufferUsage::Static)
            .ok()
            .unwrap();

        /*
         * Create Mesh
         */
        let mut mesh = vertex_buffer.into_mesh();

        mesh.set_backface_culling(false);

        /*
         * Generate Index Buffer
         */
        let mut indexes = vec![];
        let result = triangulate(
            &destination_points
                .iter()
                .map(|(x, y)| Point {
                    x: *x as f64,
                    y: *y as f64,
                })
                .collect::<Vec<Point>>(),
        );
        for index in result.triangles.iter() {
            indexes.push(*index as u32);
        }

        let index_buffer = IndexBuffer::new(ctx, &*indexes).ok().unwrap();

        mesh.set_index_buffer(index_buffer);

        let mut texture = canvas.texture().clone();
        texture.set_filter_mode(ctx, FilterMode::Linear);
        mesh.set_texture(texture);

        Option::from(mesh)
    }

    fn get_sprite_mesh(&mut self, ctx: &mut Context, layer: &Layer) -> Mesh {
        let sprite = &self.bsv3.sprites[layer.sprite as usize];

        let width = self.bsv3.sprites[layer.sprite as usize].width as f32;
        let height = self.bsv3.sprites[layer.sprite as usize].height as f32;

        let points = [
            Point { x: 0.0, y: 0.0 },
            Point {
                x: width as f64,
                y: 0.0,
            },
            Point {
                x: width as f64,
                y: height as f64,
            },
            Point {
                x: 0.0,
                y: height as f64,
            },
        ];
        /*
         * Generate Vertex Buffer
         */
        let vertices = vec![
            Vertex {
                position: Vec2::new(0.0, 0.0),
                uv: Vec2::new(
                    sprite.x as f32 / self.texture.width() as f32,
                    sprite.y as f32 / self.texture.height() as f32,
                ),
                color: Color::WHITE,
            },
            Vertex {
                position: Vec2::new(width, 0.0),
                uv: Vec2::new(
                    (sprite.x + sprite.width) as f32 / self.texture.width() as f32,
                    sprite.y as f32 / self.texture.height() as f32,
                ),
                color: Color::WHITE,
            },
            Vertex {
                position: Vec2::new(width, height),
                uv: Vec2::new(
                    (sprite.x + sprite.width) as f32 / self.texture.width() as f32,
                    (sprite.y + sprite.height) as f32 / self.texture.height() as f32,
                ),
                color: Color::WHITE,
            },
            Vertex {
                position: Vec2::new(0.0, height),
                uv: Vec2::new(
                    sprite.x as f32 / self.texture.width() as f32,
                    (sprite.y + sprite.height) as f32 / self.texture.height() as f32,
                ),
                color: Color::WHITE,
            },
        ];
        // println!("Vertices: {:#?}", vertices);
        let vertex_buffer = VertexBuffer::with_usage(ctx, &*vertices, BufferUsage::Static)
            .ok()
            .unwrap();

        /*
         * Create Mesh
         */
        let mut mesh = vertex_buffer.into_mesh();

        mesh.set_backface_culling(false);

        /*
         * Generate Index Buffer
         */
        let mut indexes = vec![];
        let result = triangulate(&points);
        for index in result.triangles.iter() {
            indexes.push(*index as u32);
        }

        let index_buffer = IndexBuffer::new(ctx, &*indexes).ok().unwrap();

        mesh.set_index_buffer(index_buffer);

        mesh.set_texture(self.texture.clone());

        mesh
    }

    fn draw_layer(&mut self, ctx: &mut Context, layer: &Layer) -> Option<(Mesh, Mat4<f32>)> {
        let sprite = &self.bsv3.sprites[layer.sprite as usize];
        let width = self.bsv3.sprites[layer.sprite as usize].width as f32;
        let height = self.bsv3.sprites[layer.sprite as usize].height as f32;

        // let mut sprite_canvas = Canvas::new(ctx, width as i32, height as i32).unwrap();
        // sprite_canvas.set_filter_mode(ctx, FilterMode::Linear);
        // graphics::set_canvas(ctx, &sprite_canvas);
        // graphics::clear(ctx, Color::rgba(0.0, 0.0, 0.0, 0.0));

        let width = self.bsv3.sprites[layer.sprite as usize].width as f32;
        let height = self.bsv3.sprites[layer.sprite as usize].height as f32;

        // Setup offset fixes for positioning when scaled/skewed
        let mut offset_x = 0.0;
        let mut offset_y = 0.0;

        if layer.scale_x < 0.0 {
            offset_x += width * abs(layer.scale_x);
        }
        if layer.skew_h < 0.0 {
            offset_x += height * abs(layer.skew_h);
        }
        if layer.scale_y < 0.0 {
            offset_y += height * abs(layer.scale_y);
        }
        if layer.skew_v < 0.0 {
            offset_y += width * abs(layer.skew_v);
        }

        let mesh = self.get_sprite_mesh(ctx, &layer);

        let transform = Mat4::new(
            layer.scale_x,
            layer.skew_h,
            0.0,
            layer.x + offset_x + CANVAS_HALF,
            layer.skew_v,
            layer.scale_y,
            0.0,
            layer.y + offset_y + CANVAS_HALF,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );

        Option::from((mesh, transform))
    }

    fn sprite_index_name(&self, layer: &Layer) -> String {
        // sprite_id | x | y | scale_x | skew_h | skew_v | scale_y | alpha
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            layer.sprite,
            layer.x,
            layer.y,
            layer.scale_x,
            layer.skew_h,
            layer.skew_v,
            layer.scale_y,
            layer.alpha
        )
    }

    fn get_index(&self) -> usize {
        let start = self.bsv3.animations[self.animation].start as usize;
        let end = self.bsv3.animations[self.animation].end as usize;
        let mut index = start;
        if start != end {
            index = start + self.timer % (end - start);
        }

        index
    }

    fn draw(&mut self, ctx: &mut Context) -> Canvas {
        let canvas_width = CANVAS_SIZE;
        let canvas_height = CANVAS_SIZE;
        let mut canvas = Canvas::new(ctx, canvas_width as i32, canvas_height as i32).unwrap();
        canvas.set_filter_mode(ctx, FilterMode::Linear);
        graphics::set_canvas(ctx, &canvas);
        if GREEN_BG {
            graphics::clear(ctx, Color::rgba(0.4, 0.7333, 0.4, 0.0));
        } else {
            graphics::clear(ctx, Color::rgba(0.5, 0.5, 0.5, 0.0));
        }
        graphics::set_canvas(ctx, &canvas);

        let index = self.get_index();

        self.timer += 1;

        if self.bsv3.format == 0x0303 {
            // loop over groups as they contain the frames for each animation frame
            let mut frames = self.bsv3.groups[index].frames.clone();
            frames.reverse();

            for frame in frames.iter() {
                self.draw_frame(ctx, &canvas, self.bsv3.frames[*frame].clone());
            }
        } else {
            self.draw_frame(ctx, &canvas, self.bsv3.frames[index].clone());
        }
        graphics::reset_canvas(ctx);
        graphics::reset_blend_state(ctx);

        canvas
    }

    fn draw_frame(&mut self, ctx: &mut Context, canvas: &Canvas, frame: Frame) -> Option<()> {
        let layers = frame.layers;
        for layer in layers.iter() {
            let layer_index = self.sprite_index_name(layer);
            let (mesh, transform) = if let Some((mesh, transform)) = self.frames.get(&layer_index) {
                (mesh.clone(), transform.clone())
            } else {
                match self.draw_layer(ctx, layer) {
                    Some((layer_mesh, layer_transform)) => {
                        self.frames
                            .insert(layer_index, (layer_mesh.clone(), layer_transform.clone()));
                        (layer_mesh, layer_transform)
                    }
                    None => continue,
                }
            };

            let offset_transform = transform
                + Mat4::new(
                    0.0,
                    0.0,
                    0.0,
                    self.offset_x,
                    0.0,
                    0.0,
                    0.0,
                    self.offset_y,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                );

            let alpha = layer.alpha as f32 / 255.0;

            graphics::set_blend_state(ctx, BlendState::alpha(true));

            graphics::set_transform_matrix(ctx, offset_transform);
            mesh.draw(
                ctx,
                DrawParams::default().color(Color::rgba(alpha, alpha, alpha, alpha)),
            );

            graphics::reset_transform_matrix(ctx);
        }

        Some(())
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if input::is_mouse_scrolled_up(ctx) {
            self.scene.scale.x += 0.25;
            self.scene.scale.y += 0.25;
        } else if input::is_mouse_scrolled_down(ctx) {
            self.scene.scale.x -= 0.25;
            self.scene.scale.y -= 0.25;
        }

        if self.scene.scale.x < 0.25 || self.scene.scale.y < 0.25 {
            self.scene.scale.x = 0.25;
            self.scene.scale.y = 0.25;
        } else if self.scene.scale.x > 4.0 || self.scene.scale.y > 4.0 {
            self.scene.scale.x = 4.0;
            self.scene.scale.y = 4.0;
        }

        self.clip_canvas = self.scene.draw(ctx);

        if SAVE_CANVAS {
            self.save_canvas(ctx);
        }

        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> tetra::Result {
        // println!("{:?}", event);

        if let Event::FileDropped { ref path } = event {
            let mut file_path = path.to_str().unwrap().to_string();

            if file_path.ends_with(".rgb") {
                // check for bsv3 file existing with same name
                let bsv3_file = file_path.replace(".rgb", ".bsv3");
                let rgb_file = file_path.replace(".bsv3", ".rgb");
                if !std::path::Path::new(&bsv3_file).exists() {
                    return Ok(());
                }

                file_path = bsv3_file;
            }

            if !file_path.ends_with(".bsv3") {
                return Ok(()); // Ignore non bsv3 files.
            }

            if !std::path::Path::new(&file_path.replace(".bsv3", ".rgb")).exists() {
                return Ok(());
            }

            self.scene = Scene::new(ctx, &file_path, self.font.clone())?;
        }

        if let Event::MouseButtonPressed { button, .. } = event {
            if button == input::MouseButton::Left {
                self.mouse_down = true;
            }
        }

        if let Event::MouseButtonReleased { button, .. } = event {
            if button == input::MouseButton::Left {
                self.mouse_down = false;
            }
            if button == input::MouseButton::Right {
                self.scene.timer = 0;
                self.scene.animation =
                    (self.scene.animation + 1) % self.scene.bsv3.animations.len();
                println!("Animation: {}", self.scene.animation);
            }
        }

        if let Event::MouseMoved { position, delta } = event {
            if self.mouse_down {
                self.scene.offset_x += delta.x * (1.0 / self.scene.scale.x);
                self.scene.offset_y += delta.y * (1.0 / self.scene.scale.y);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        if GREEN_BG {
            graphics::clear(ctx, Color::rgb(0.4, 0.7333333333333, 0.4));
        } else {
            graphics::clear(ctx, Color::rgb(0.5, 0.5, 0.5));
        }

        self.clip_canvas.draw(
            ctx,
            DrawParams::default()
                .position(Vec2::new(
                    (WINDOW_WIDTH - CANVAS_SIZE * self.scene.scale.x) / 2.0,
                    (WINDOW_HEIGHT - CANVAS_SIZE * self.scene.scale.y) / 2.0,
                ))
                .scale(self.scene.scale),
        );

        /*
         * Draw Timer
         */
        let mut text_steps = Text::new(format!("Time: {}", self.scene.timer), self.font.clone());
        text_steps.draw(ctx, Vec2::new(10.0, 10.0));

        /*
         * Draw FPS
         */
        let mut text_steps = Text::new(
            format!("FPS: {}", time::get_fps(ctx).round()),
            self.font.clone(),
        );
        text_steps.draw(ctx, Vec2::new(10.0, 30.0));

        /*
         * Draw Animation FPS
         */
        let frame_rate = if let Timestep::Fixed(time_step) = time::get_timestep(ctx) {
            time_step
        } else {
            1000f64
        };
        let mut text_steps = Text::new(format!("APS: {}", frame_rate), self.font.clone());
        text_steps.draw(ctx, Vec2::new(10.0, 50.0));

        /*
         * Draw Scale
         */
        let mut text_scale = Text::new(format!("Scale: {}", self.scene.scale.x), self.font.clone());
        text_scale.draw(ctx, Vec2::new(10.0, 80.0));

        /*
         * Draw Animation
         */
        let mut text = String::from("Animations");
        // loop over animations and list them put current one in brackets
        for (index, animation) in self.scene.bsv3.animations.iter().enumerate() {
            if index == self.scene.animation {
                text = format!(
                    "{}\n({:03} - {:03}) [{}] {}",
                    text, animation.start, animation.end, index, animation.name
                );
            } else {
                text = format!(
                    "{}\n({:03} - {:03}) {} {}",
                    text, animation.start, animation.end, index, animation.name
                );
            }
        }

        let mut text_scale = Text::new(text, self.font.clone());
        text_scale.draw(ctx, Vec2::new(10.0, 110.0));

        Ok(())
    }
}
