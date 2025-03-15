#![allow(unused_variables)]
#![allow(dead_code)]

use colored::Colorize;
use std::cmp::max;

use crate::ea::file_buffer::FileBuffer;

static SAVE_RGB: bool = false;

#[derive(Debug, Clone)]
pub struct Sprite {
    pub name: String,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: u16,
    pub sprite: u16,
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub skew_h: f32,
    pub skew_v: f32,
    pub scale_y: f32,
    pub alpha: u8,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub layer_count: u16,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone)]
pub struct FrameGroup {
    pub frame_count: u16,
    pub frames: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub start: u16,
    pub end: u16,
}

#[derive(Debug)]
pub struct BSV3 {
    pub file: FileBuffer,
    pub format: u16,
    pub data_type: u8,
    pub image_name: String,
    pub sprites: Vec<Sprite>,
    pub frames: Vec<Frame>,
    pub groups: Vec<FrameGroup>,
    pub animations: Vec<Animation>,
}

impl BSV3 {
    pub fn new(name: String) -> BSV3 {
        let file = FileBuffer::new(name.clone());

        BSV3 {
            file,
            format: 0,
            data_type: 0,
            image_name: String::from(""),
            sprites: vec![],
            frames: vec![],
            groups: vec![],
            animations: vec![],
        }
    }

    pub fn parse(&mut self) {
        let start_time = std::time::Instant::now();
        print!("{} {}.bsv3\n", "Parsing".blue(), self.file.name);

        self.format = self.file.read_uint_16();

        println!("Format: {}", self.format);

        if self.format == 0x0104 {
            self.file.skip(3);
        }

        if self.format == 0x0303 {
            self.file.read_float_32();
        }

        let sprite_count = self.file.read_uint_16();

        println!("Sprite count: {}", sprite_count);

        self.data_type = self.file.read_uint_8();

        println!("Data type: {}", self.data_type);

        if self.format == 0x0203 {
            self.image_name = self.file.read_string_8();
            println!("Image name: {}", self.image_name);
        }

        let sprites = (0..sprite_count)
            .map(|_| Sprite {
                name: self.file.read_string_8(),
                x: self.file.read_uint_16(),
                y: self.file.read_uint_16(),
                width: max(self.file.read_uint_16(), 1),
                height: max(self.file.read_uint_16(), 1),
            })
            .collect::<Vec<Sprite>>();

        for i in 0..sprite_count as usize {
            self.sprites.insert(i, sprites[i].clone());
        }

        if self.format == 0x0103 || self.format == 0x0203 {
            let frame_count = self.file.read_uint_16();

            println!("Frame count: {}", frame_count);

            let frames = (0..frame_count)
                .map(|_| {
                    let layer_count = self.file.read_uint_16();
                    // println!("Layer count: {}", layer_count);
                    self.file.skip(1);
                    let mut layers = (0..layer_count)
                        .map(|n| {
                            let sprite = self.file.read_uint_16();
                            let x = self.file.read_float_32();
                            let y = self.file.read_float_32();
                            let scale_x = self.file.read_float_32();
                            let skew_v = self.file.read_float_32();
                            let skew_h = self.file.read_float_32();
                            let scale_y = self.file.read_float_32();

                            let alpha = if self.data_type == 1 {
                                self.file.read_uint_8()
                            } else {
                                255
                            };

                            Layer {
                                id: n,
                                sprite,
                                x,
                                y,
                                scale_x,
                                skew_h,
                                skew_v,
                                scale_y,
                                alpha,
                            }
                        })
                        .collect::<Vec<Layer>>();

                    layers.reverse();

                    Frame {
                        layer_count,
                        layers,
                    }
                })
                .collect::<Vec<Frame>>();

            for i in 0..frame_count as usize {
                self.frames.insert(i, frames[i].clone());
            }
        } else if self.format == 0x0303 {
            let group_count = self.file.read_uint_16();

            println!("Group count: {}", group_count);

            let frame_count = self.file.read_uint_16();

            println!("Frame count: {}", frame_count);

            let frames = (0..frame_count)
                .map(|_| {
                    let sprite = self.file.read_uint_16();
                    let x = self.file.read_float_32();
                    let y = self.file.read_float_32();
                    let scale_x = self.file.read_float_32();
                    let skew_v = self.file.read_float_32();
                    let skew_h = self.file.read_float_32();
                    let scale_y = self.file.read_float_32();

                    let alpha = if self.data_type == 1 {
                        self.file.read_uint_8()
                    } else {
                        255
                    };

                    Frame {
                        layer_count: 1,
                        layers: vec![Layer {
                            id: 0,
                            sprite,
                            x,
                            y,
                            scale_x,
                            skew_h,
                            skew_v,
                            scale_y,
                            alpha,
                        }],
                    }
                })
                .collect::<Vec<Frame>>();

            for i in 0..frame_count as usize {
                self.frames.insert(i, frames[i].clone());
            }

            let groups = (0..group_count)
                .map(|_| {
                    let frame_count = self.file.read_uint_16();
                    self.file.read_uint_8();
                    let frames = (0..frame_count)
                        .map(|_| self.file.read_uint_16() as usize)
                        .collect::<Vec<usize>>();

                    FrameGroup {
                        frame_count,
                        frames,
                    }
                })
                .collect::<Vec<FrameGroup>>();

            for i in 0..group_count as usize {
                self.groups.insert(i, groups[i].clone());
            }
        }

        let animation_count = self.file.read_uint_16();

        println!("Animation count: {}", animation_count);

        let animations = (0..animation_count)
            .map(|_| Animation {
                name: self.file.read_string_8(),
                start: self.file.read_uint_16(),
                end: self.file.read_uint_16(),
            })
            .collect::<Vec<Animation>>();

        for i in 0..animation_count as usize {
            self.animations.insert(i, animations[i].clone());
        }

        // println!("Animations: {:#?}", self.animations);

        let end_time = std::time::Instant::now();
        print!("{} {:?}\n", "Done in".green(), end_time - start_time);
    }
}

// https://github.com/al1sant0s/tstorgb/blob/main/src/tstorgb/parsers/bsv3.py
// https://github.com/al1sant0s/tstorgb/blob/main/src/tstorgb/parsers/addons/bsv3_addon.py
