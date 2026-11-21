use std::path::PathBuf;
use std::sync::OnceLock;

use tauri::utils::platform::resource_dir;
use tauri::Context;

static RESOURCE_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

pub fn init(context: &Context<tauri::Wry>) {
    let dir = resource_dir(context.package_info(), &Default::default()).ok();
    RESOURCE_DIR
        .set(dir)
        .expect("resources::init must only be called once");
}

fn installed_dir() -> Option<&'static PathBuf> {
    RESOURCE_DIR.get().and_then(|dir| dir.as_ref())
}

pub fn bundled_resource(relative: &str) -> Option<PathBuf> {
    installed_dir()
        .map(|dir| dir.join("resources").join(relative))
        .filter(|path| path.exists())
        .or_else(|| {
            let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join(relative);
            dev.exists().then_some(dev)
        })
}

pub fn installed_resource_dir() -> Option<PathBuf> {
    installed_dir().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_resource_finds_dev_includes() {
        let path = bundled_resource("includes").expect("dev includes dir should exist");
        assert!(path.join("testlib.h").exists());
    }

    #[test]
    fn bundled_resource_missing_relative_returns_none() {
        assert!(bundled_resource("does-not-exist").is_none());
    }
}
