use std::path::PathBuf;

use tauri::utils::platform::resource_dir;

pub struct BinaryResolver;

impl BinaryResolver {
    fn get_resource_dir() -> Option<PathBuf> {
        let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
        resource_dir(context.package_info(), &Default::default()).ok()
    }

    fn get_platform_string() -> &'static str {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return "linux-x64";

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return "darwin-x64";

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return "darwin-arm64";

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return "windows-x64";

        #[cfg(not(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "windows", target_arch = "x86_64")
        )))]
        compile_error!("Unsupported platform");
    }

    pub fn get_bundled_binary(name: &str) -> Option<PathBuf> {
        let resource_dir = Self::get_resource_dir()?;

        let binary_name = if cfg!(windows) {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };

        let platform = Self::get_platform_string();
        let binary_path = resource_dir
            .join("binaries")
            .join(platform)
            .join(&binary_name);

        if binary_path.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(&binary_path) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = std::fs::set_permissions(&binary_path, perms);
                }
            }

            println!("Found bundled {}: {:?}", name, binary_path);
            Some(binary_path)
        } else {
            println!("Bundled {} not found at {:?}", name, binary_path);
            None
        }
    }
}
