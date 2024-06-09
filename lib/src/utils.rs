use crate::config::Settings;
use anyhow::{anyhow, Context, Result};
use pinyin::ToPinyin;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::{
    ffi::OsStr,
    process::{Child, Command},
};
use unicode_segmentation::UnicodeSegmentation;

pub fn get_pin_yin(input: &str) -> String {
    let mut b = String::new();
    for (index, f) in input.to_pinyin().enumerate() {
        match f {
            Some(p) => {
                b.push_str(p.plain());
            }
            None => {
                if let Some(c) = input.to_uppercase().chars().nth(index) {
                    b.push(c);
                }
            }
        }
    }
    b
}

pub fn filetype_supported(current_node: &str) -> bool {
    let p = Path::new(current_node);

    if p.starts_with("http") {
        return true;
    }

    #[cfg(any(feature = "mpv", feature = "gst"))]
    if let Some(ext) = p.extension() {
        if ext == "opus" {
            return true;
        }
        if ext == "aiff" {
            return true;
        }
    }

    match p.extension() {
        Some(ext) if ext == "mkv" || ext == "mka" => true,
        Some(ext) if ext == "mp3" => true,
        // Some(ext) if ext == "aiff" => true,
        Some(ext) if ext == "flac" => true,
        Some(ext) if ext == "m4a" => true,
        Some(ext) if ext == "aac" => true,
        // Some(ext) if ext == "opus" => true,
        Some(ext) if ext == "ogg" => true,
        Some(ext) if ext == "wav" => true,
        Some(ext) if ext == "webm" => true,
        Some(_) | None => false,
    }
}

pub fn is_playlist(current_node: &str) -> bool {
    let p = Path::new(current_node);

    match p.extension() {
        Some(ext) if ext == "m3u" => true,
        Some(ext) if ext == "m3u8" => true,
        Some(ext) if ext == "pls" => true,
        Some(ext) if ext == "asx" => true,
        Some(ext) if ext == "xspf" => true,
        Some(_) | None => false,
    }
}

pub fn get_parent_folder(filename: &str) -> String {
    let parent_folder: PathBuf;
    let path_old = Path::new(filename);

    if path_old.is_dir() {
        parent_folder = path_old.to_path_buf();
        return parent_folder.to_string_lossy().to_string();
    }
    match path_old.parent() {
        Some(p) => parent_folder = p.to_path_buf(),
        None => parent_folder = std::env::temp_dir(),
    }
    parent_folder.to_string_lossy().to_string()
}

pub fn get_app_config_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().ok_or_else(|| anyhow!("failed to find os config dir."))?;
    path.push("termusic");

    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }
    Ok(path)
}

/// Get the podcast directoy resolved and created
fn get_podcast_save_path(config: &Settings) -> Result<PathBuf> {
    let full_path = shellexpand::path::tilde(&config.podcast_dir);
    if !full_path.exists() {
        std::fs::create_dir_all(&full_path)?;
    }
    Ok(full_path.into_owned())
}

/// Get the download directory for the provided `pod_title` and create it if not existing
pub fn create_podcast_dir(config: &Settings, pod_title: String) -> Result<PathBuf> {
    let mut download_path = get_podcast_save_path(config).context("get podcast directory")?;
    download_path.push(pod_title);
    std::fs::create_dir_all(&download_path).context("creating podcast download directory")?;

    Ok(download_path)
}

pub fn playlist_get_vec(current_node: &str) -> Result<Vec<String>> {
    let p = Path::new(current_node);
    let p_base = absolute_path(p.parent().ok_or_else(|| anyhow!("cannot find path root"))?)?;
    let str = std::fs::read_to_string(p)?;
    let items =
        crate::playlist::decode(&str).map_err(|e| anyhow!("playlist decode error: {}", e))?;
    let mut vec = vec![];
    for mut item in items {
        item.absoluteize(&p_base);

        // TODO: refactor to return better values
        vec.push(item.to_string());
    }
    Ok(vec)
}

/// Some helper functions for dealing with Unicode strings.
#[allow(clippy::module_name_repetitions)]
pub trait StringUtils {
    fn substr(&self, start: usize, length: usize) -> String;
    fn grapheme_len(&self) -> usize;
}

impl StringUtils for String {
    /// Takes a slice of the String, properly separated at Unicode
    /// grapheme boundaries. Returns a new String.
    fn substr(&self, start: usize, length: usize) -> String {
        return self
            .graphemes(true)
            .skip(start)
            .take(length)
            .collect::<String>();
    }

    /// Counts the total number of Unicode graphemes in the String.
    fn grapheme_len(&self) -> usize {
        return self.graphemes(true).count();
    }
}

/// Spawn a detached process
/// # Panics
/// panics when spawn server failed
pub fn spawn_process<A: IntoIterator<Item = S> + Clone, S: AsRef<OsStr>>(
    prog: &Path,
    superuser: bool,
    shout_output: bool,
    args: A,
) -> std::io::Result<Child> {
    let mut cmd = if superuser {
        let mut cmd_t = Command::new("sudo");
        cmd_t.arg(prog);
        cmd_t
    } else {
        Command::new(prog)
    };
    if !shout_output {
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
    }

    cmd.args(args);
    cmd.spawn()
}

/// Absolutize a given path with the current working directory.
///
/// This function, unlike [`std::fs::canonicalize`] does *not* hit the filesystem and so does not require the input path to exist yet.
///
/// Examples:
/// `./somewhere` -> `/absolute/./somewhere`
/// `.\somewhere` -> `C:\somewhere`
///
/// in the future consider replacing with [`std::path::absolute`] once stable
pub fn absolute_path(path: &Path) -> std::io::Result<Cow<'_, Path>> {
    if path.is_absolute() {
        Ok(Cow::Borrowed(path))
    } else {
        Ok(Cow::Owned(std::env::current_dir()?.join(path)))
    }
}

/// Absolutize a given path with the given base.
///
/// `base` is expected to be absoulte!
///
/// This function, unlike [`std::fs::canonicalize`] does *not* hit the filesystem and so does not require the input path to exist yet.
///
/// Examples:
/// `./somewhere` -> `/absolute/./somewhere`
/// `.\somewhere` -> `C:\somewhere`
///
/// in the future consider replacing with [`std::path::absolute`] once stable
pub fn absolute_path_base<'a>(path: &'a Path, base: &Path) -> Cow<'a, Path> {
    if path.is_absolute() {
        Cow::Borrowed(path)
    } else {
        Cow::Owned(base.join(path))
    }
}

#[cfg(test)]
#[allow(clippy::non_ascii_literal)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pin_yin() {
        assert_eq!(get_pin_yin("陈一发儿"), "chenyifaer".to_string());
        assert_eq!(get_pin_yin("Gala乐队"), "GALAledui".to_string());
        assert_eq!(get_pin_yin("乐队Gala乐队"), "leduiGALAledui".to_string());
        assert_eq!(get_pin_yin("Annett Louisan"), "ANNETT LOUISAN".to_string());
    }
}
