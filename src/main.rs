use std::env;
use std::fs;
use image::RgbaImage;
use image::Rgba;

// My first time trying out rust
// This program takes in a source filepath and a destination filepath
// The source should be a qoi image and the destination can be any image supported by the image crate
// https://qoiformat.org/
// https://github.com/image-rs/image/blob/main/README.md
fn main() -> std::io::Result<()>{

    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 3, "Wrong number of arguments have been provided. Expecting {}, provided {}", 2, args.len());

    let data : Vec<u8> = fs::read(&args[1])?;

    assert!(data[0] == b'q', "Magic bytes not present");
    assert!(data[1] == b'o', "Magic bytes not present");
    assert!(data[2] == b'i', "Magic bytes not present");
    assert!(data[3] == b'f', "Magic bytes not present");

    let width : u32 = (data[4] as u32) << 24 | (data[5] as u32) << 16 | (data[6] as u32) << 8 | (data[7] as u32);
    let height : u32 = (data[8] as u32) << 24 | (data[9] as u32) << 16 | (data[10] as u32) << 8 | (data[11] as u32);
    let channels = data[12];
    let colorspace = data[13];

    println!("Width: {}", width);
    println!("Height: {}", height);
    println!("Channels: {}", channels);
    println!("Colorspace: {}", if colorspace == 0 { "sRGB" } else { "linear" });

    let mut cursor : usize = 14;
    let mut last_pixel = Rgba([0, 0, 0, 255]);
    let mut running_pixels : [Rgba<u8>; 64] = [Rgba([0, 0, 0, 0]); 64];

    let mut img = RgbaImage::new(width, height);
    let mut run = 0;

    for y in 0..height
    {
        for x in 0..width
        {
            
            if run > 0
            {
                img.put_pixel(x, y, last_pixel);
                run -= 1;
                continue;
            }
            let byte = data[cursor];

            if byte == 0b11111110
            {
                // QOI_OP_RGB
                let r = data[cursor + 1];
                let g = data[cursor + 2];
                let b = data[cursor + 3];
                let a = last_pixel[3];
                last_pixel = Rgba([r, g, b, a]);
                img.put_pixel(x, y, last_pixel);
                cursor += 4;
            }
            else if byte == 0b11111111
            {
                // QOI_OP_RGBA
                let r = data[cursor + 1];
                let g = data[cursor + 2];
                let b = data[cursor + 3];
                let a = data[cursor + 4];
                last_pixel = Rgba([r, g, b, a]);
                img.put_pixel(x, y, last_pixel);
                cursor += 5;
            }
            else if byte & 0b11000000 == 0b00000000
            {
                // QOI_OP_INDEX
                let index = byte & 0x3f;
                last_pixel = running_pixels[index as usize];
                img.put_pixel(x, y, last_pixel);
                cursor += 1;
            }
            else if byte & 0b11000000 == 0b01000000
            {
                // QOI_OP_DIF
                let dr : u8 = (byte >> 4) & 0x03;
                let dg : u8 = (byte >> 2) & 0x03;
                let db : u8 = (byte >> 0) & 0x03;
                last_pixel[0] = last_pixel[0].wrapping_add(dr).wrapping_sub(2);
                last_pixel[1] = last_pixel[1].wrapping_add(dg).wrapping_sub(2);
                last_pixel[2] = last_pixel[2].wrapping_add(db).wrapping_sub(2);
                img.put_pixel(x, y, last_pixel);
                cursor += 1;
            }
            else if byte & 0b11000000 == 0b10000000
            {
                // QOI_OP_LUMA
                let dg : u8 = (byte >> 0) & 0x3f;
                let drdg : u8 = (data[cursor+1] >> 4) & 0x0f;
                let dbdg : u8 = (data[cursor+1] >> 0) & 0x0f;

                last_pixel[0] = last_pixel[0].wrapping_add(dg).wrapping_sub(32);
                last_pixel[1] = last_pixel[1].wrapping_add(dg).wrapping_sub(32);
                last_pixel[2] = last_pixel[2].wrapping_add(dg).wrapping_sub(32);

                last_pixel[0] = last_pixel[0].wrapping_add(drdg).wrapping_sub(8);
                last_pixel[2] = last_pixel[2].wrapping_add(dbdg).wrapping_sub(8);
                
                img.put_pixel(x, y, last_pixel);
                cursor += 2;
            }
            else if byte & 0b11000000 == 0b11000000
            {
                // QOI_OP_RUN
                let run_length = byte & 0x3f;
                run = run_length;
                img.put_pixel(x, y, last_pixel);
                cursor += 1;
            }

            let index = (last_pixel[0] as u16 * 3 + last_pixel[1] as u16 * 5 + last_pixel[2] as u16 * 7 + last_pixel[3] as u16 * 11) % 64;

            running_pixels[index as usize] = last_pixel;

        }
    }

    assert!(data[cursor] == 0x00, "End marker not found");
    assert!(data[cursor + 1] == 0x00, "End marker not found");
    assert!(data[cursor + 2] == 0x00, "End marker not found");
    assert!(data[cursor + 3] == 0x00, "End marker not found");
    assert!(data[cursor + 4] == 0x00, "End marker not found");
    assert!(data[cursor + 5] == 0x00, "End marker not found");
    assert!(data[cursor + 6] == 0x00, "End marker not found");
    assert!(data[cursor + 7] == 0x01, "End marker not found");

    img.save(&args[2]).unwrap();    

    return Ok(());
}
