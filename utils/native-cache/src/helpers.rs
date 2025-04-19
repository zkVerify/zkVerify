use std::{ffi::OsStr, path::Path};

#[macro_export]
macro_rules! log {
    ($($tokens: tt)*) => {
        if $crate::ENABLE_LOGS {
            println!("cargo:warning={}", format!($($tokens)*))
        }
    }
}

/// Identify if the current path is a static or dynamic lib
pub fn is_dyn_or_static_lib(p: impl AsRef<Path>) -> bool {
    let p = p.as_ref();
    if let Some((name, ext)) = p
        .file_name()
        .and_then(OsStr::to_str)
        .and_then(|n| p.extension().map(|e| (n, e)))
    {
        (ext == "a" || ext == "so" || ext == "dylib") && name.starts_with("lib")
    } else {
        false
    }
}

/// check if the file has the given name
pub fn is_name(p: impl AsRef<Path>, name: &str) -> bool {
    p.as_ref().file_name().and_then(OsStr::to_str) == Some(name)
}
