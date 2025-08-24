use png::ColorType;
use std::fs::File;
use std::io::BufWriter;

pub fn test_png_save() -> Result<(), Box<dyn std::error::Error>> {
    // Create test frame data - 405x720 RGBA (mobile aspect ratio)
    let width = 405u32;
    let height = 720u32;
    let frame_size = (width * height * 4) as usize;
    let mut frame_data = vec![0u8; frame_size];
    
    // Create a simple pattern - red/green gradient
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            frame_data[idx] = (255.0 * (x as f32 / width as f32)) as u8; // R - horizontal gradient
            frame_data[idx + 1] = (255.0 * (y as f32 / height as f32)) as u8; // G - vertical gradient  
            frame_data[idx + 2] = 100; // B - constant blue
            frame_data[idx + 3] = 255; // A - full opacity
        }
    }
    
    // Save as PNG
    let file = File::create("test_output.png")?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&frame_data)?;
    
    println!("✅ Test PNG created successfully: test_output.png");
    println!("   Size: {}x{} ({} bytes)", width, height, frame_data.len());
    Ok(())
}