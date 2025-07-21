// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Check if we have a config.rs file in the build directory (from meson)
    let out_dir = env::var("OUT_DIR").unwrap();
    let build_config = Path::new(&out_dir).join("../../../config.rs");
    let src_config = Path::new("src/config.rs");
    
    // If meson generated a config.rs in the build directory, copy it to src/
    if build_config.exists() && (!src_config.exists() || 
        fs::metadata(&build_config).unwrap().modified().unwrap() > 
        fs::metadata(&src_config).unwrap().modified().unwrap()) {
        
        if let Ok(contents) = fs::read_to_string(&build_config) {
            let _ = fs::write(src_config, contents);
        }
    }
    
    // If no config.rs exists, create a default one for development
    if !src_config.exists() {
        let default_config = r#"// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

pub static VERSION: &str = "2024.2-dev";
pub static GETTEXT_PACKAGE: &str = "amberol";
pub static LOCALEDIR: &str = "/usr/local/share/locale";
pub static PKGDATADIR: &str = "/usr/local/share/amberol";
pub static APPLICATION_ID: &str = "io.bassi.Amberol.Devel";
pub static PROFILE: &str = "development";
"#;
        let _ = fs::write(src_config, default_config);
    }
}