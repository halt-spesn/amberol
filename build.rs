// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/icons/hicolor/scalable/apps/io.bassi.Amberol.ico");
    println!("cargo:rerun-if-changed=icon.rc");
    
    // Only generate icons for Windows builds
    if cfg!(target_os = "windows") {
        // Use appropriate linker arguments based on the target
        if cfg!(target_env = "msvc") {
            // MSVC linker - compile and link resource file
            println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
            compile_resource_file();
        } else {
            // MinGW/GNU linker - compile and link resource file
            println!("cargo:rustc-link-arg=-Wl,--subsystem,windows");
            compile_resource_file();
        }
        
        // Use the existing ICO file or generate one if it doesn't exist
        let ico_source = "data/icons/hicolor/scalable/apps/io.bassi.Amberol.ico";
        let ico_target = "amberol.ico";
        
        if Path::new(ico_source).exists() {
            // Copy the existing ICO file to the build directory
            if let Err(e) = std::fs::copy(ico_source, ico_target) {
                println!("cargo:warning=Failed to copy app icon: {}", e);
                // Fallback to generating the icon
                generate_app_icon().unwrap_or_else(|e| {
                    println!("cargo:warning=Failed to generate fallback app icon: {}", e);
                });
            } else {
                println!("cargo:warning=Using existing app icon: {}", ico_source);
            }
        } else {
            println!("cargo:warning=ICO file not found at {}, generating programmatic icon", ico_source);
            generate_app_icon().unwrap_or_else(|e| {
                println!("cargo:warning=Failed to generate app icon: {}", e);
            });
        }
    }
}

fn generate_app_icon() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple programmatic icon
    let sizes = [16, 32, 48, 256];
    let mut ico_data = Vec::new();
    
    // ICO file header
    ico_data.extend_from_slice(&[0, 0]); // Reserved (must be 0)
    ico_data.extend_from_slice(&[1, 0]); // Type (1 = ICO)
    ico_data.extend_from_slice(&(sizes.len() as u16).to_le_bytes()); // Number of images
    
    let mut image_data = Vec::new();
    let mut directory_entries = Vec::new();
    
    for &size in &sizes {
        // Create a simple bitmap for each size
        let bmp_data = create_simple_icon_bitmap(size)?;
        
        // ICO directory entry
        let mut entry = Vec::new();
        entry.push(if size == 256 { 0 } else { size as u8 }); // Width (0 = 256)
        entry.push(if size == 256 { 0 } else { size as u8 }); // Height (0 = 256)
        entry.push(0); // Color palette (0 = no palette)
        entry.push(0); // Reserved
        entry.extend_from_slice(&1u16.to_le_bytes()); // Color planes
        entry.extend_from_slice(&32u16.to_le_bytes()); // Bits per pixel
        entry.extend_from_slice(&(bmp_data.len() as u32).to_le_bytes()); // Image size
        entry.extend_from_slice(&((6 + sizes.len() * 16 + image_data.len()) as u32).to_le_bytes()); // Image offset
        
        directory_entries.extend_from_slice(&entry);
        image_data.extend_from_slice(&bmp_data);
    }
    
    // Combine header + directory + images
    ico_data.extend_from_slice(&directory_entries);
    ico_data.extend_from_slice(&image_data);
    
    // Write to file
    std::fs::write("amberol.ico", ico_data)?;
    println!("Generated amberol.ico");
    
    Ok(())
}

fn create_simple_icon_bitmap(size: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a simple bitmap with the Amberol-style music note
    let mut bmp_data = Vec::new();
    
    // BMP header for 32-bit RGBA
    let file_size = 54 + (size * size * 4); // Header + pixel data
    
    // BMP file header
    bmp_data.extend_from_slice(b"BM"); // Signature
    bmp_data.extend_from_slice(&(file_size as u32).to_le_bytes()); // File size
    bmp_data.extend_from_slice(&[0, 0, 0, 0]); // Reserved
    bmp_data.extend_from_slice(&54u32.to_le_bytes()); // Offset to pixel data
    
    // BMP info header
    bmp_data.extend_from_slice(&40u32.to_le_bytes()); // Header size
    bmp_data.extend_from_slice(&(size as i32).to_le_bytes()); // Width
    bmp_data.extend_from_slice(&(-(size as i32)).to_le_bytes()); // Height (negative for top-down)
    bmp_data.extend_from_slice(&1u16.to_le_bytes()); // Planes
    bmp_data.extend_from_slice(&32u16.to_le_bytes()); // Bits per pixel
    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Compression
    bmp_data.extend_from_slice(&(size * size * 4).to_le_bytes()); // Image size
    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // X pixels per meter
    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Y pixels per meter
    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Colors used
    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Important colors
    
    // Create pixel data - simple music note shape
    for y in 0..size {
        for x in 0..size {
            let (r, g, b, a) = if is_music_note_pixel(x, y, size) {
                (255, 140, 0, 255) // Orange color
            } else {
                (0, 0, 0, 0) // Transparent
            };
            
            // BMP uses BGRA format
            bmp_data.push(b);
            bmp_data.push(g);
            bmp_data.push(r);
            bmp_data.push(a);
        }
    }
    
    Ok(bmp_data)
}

fn is_music_note_pixel(x: u32, y: u32, size: u32) -> bool {
    let fx = x as f32 / size as f32;
    let fy = y as f32 / size as f32;
    
    // Simple music note shape
    // Note head (circle)
    let head_x: f32 = 0.3;
    let head_y: f32 = 0.7;
    let head_radius: f32 = 0.15;
    
    if (fx - head_x).powi(2) + (fy - head_y).powi(2) <= head_radius.powi(2) {
        return true;
    }
    
    // Note stem (vertical line)
    if fx >= head_x + head_radius - 0.05 && fx <= head_x + head_radius + 0.05 && fy >= 0.2 && fy <= head_y {
        return true;
    }
    
    // Note flag (curved)
    if fx >= head_x + head_radius && fx <= 0.8 && fy >= 0.2 && fy <= 0.4 {
        let curve: f32 = 0.3 + 0.2 * ((fx - head_x - head_radius) * 5.0).sin();
        if fy <= curve {
            return true;
        }
    }
    
    false
}

/// Compile Windows resource file to embed icon and version info
fn compile_resource_file() {
    let rc_file = "icon.rc";
    
    if !Path::new(rc_file).exists() {
        println!("cargo:warning=Resource file {} not found, skipping", rc_file);
        return;
    }
    
    // Different output files and commands for different toolchains
    let (output_file, command, args) = if cfg!(target_env = "msvc") {
        // MSVC: Use rc.exe to create .res file
        ("icon.res", "rc", vec!["/fo", "icon.res", rc_file])
    } else {
        // MinGW: Use windres to create .o file
        ("icon.o", "windres", vec!["-i", rc_file, "-o", "icon.o"])
    };
    
    // Try to compile the resource file
    let output = std::process::Command::new(command)
        .args(&args)
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                // Only add the link argument if compilation was successful
                if std::path::Path::new(output_file).exists() {
                    println!("cargo:rustc-link-arg={}", output_file);
                    println!("cargo:warning=Successfully compiled and linked resource file: {}", output_file);
                } else {
                    println!("cargo:warning=Resource compilation reported success but {} not found", output_file);
                }
            } else {
                println!("cargo:warning=Failed to compile resource file:");
                println!("cargo:warning=  Command: {} {:?}", command, args);
                println!("cargo:warning=  stderr: {}", String::from_utf8_lossy(&result.stderr));
                println!("cargo:warning=  stdout: {}", String::from_utf8_lossy(&result.stdout));
                println!("cargo:warning=Continuing build without resource file...");
            }
        }
        Err(e) => {
            println!("cargo:warning=Resource compiler '{}' not available: {}. Skipping icon embedding.", command, e);
            println!("cargo:warning=Install MinGW-w64 development tools for icon embedding support.");
        }
    }
}