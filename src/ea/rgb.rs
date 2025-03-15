pub fn rgba4444_to_rgba8888(input: Vec<u8>, premultiplied: bool) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len() * 2); // Each RGBA4444 pixel becomes 4 bytes in RGBA8888

    for chunk in input.chunks(2) {
        if chunk.len() == 2 {
            // Extract 4-bit components
            let r = (chunk[1] >> 4) & 0xF;
            let g = chunk[1] & 0xF;
            let b = (chunk[0] >> 4) & 0xF;
            let a = chunk[0] & 0xF;

            // Scale 4-bit components to 8-bit
            let mut r = (r << 4) | r;
            let mut g = (g << 4) | g;
            let mut b = (b << 4) | b;
            let a = (a << 4) | a;

            if premultiplied {
                r = (r as f32 * (a as f32 / 255.0)) as u8;
                g = (g as f32 * (a as f32 / 255.0)) as u8;
                b = (b as f32 * (a as f32 / 255.0)) as u8;
            }

            // Push the 8-bit components into the output vector
            output.push(r);
            output.push(g);
            output.push(b);
            output.push(a);
        }
    }

    output
}

pub fn la88_to_rgba8888(input: Vec<u8>, premultiplied: bool) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len() * 2); // Each LA88 pixel becomes 4 bytes in RGBA8888

    for chunk in input.chunks(2) {
        if chunk.len() == 2 {
            // Extract 8-bit components
            let l = chunk[1];
            let a = chunk[0];

            let r = l;
            let g = l;
            let b = l;

            if premultiplied {
                // let r = (l as f32 * (a as f32 / 255.0)) as u8;
                // let g = (l as f32 * (a as f32 / 255.0)) as u8;
                // let b = (l as f32 * (a as f32 / 255.0)) as u8;
            }

            // Push the 8-bit components into the output vector
            output.push(r);
            output.push(g);
            output.push(b);
            output.push(a);
        }
    }

    output
}
