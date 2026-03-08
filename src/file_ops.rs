use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use walkdir::WalkDir;
use crate::types::FileStats;
#[cfg(unix)]
extern crate libc;

// ── Read file as text ─────────────────────────────────────────────────────────
pub fn read_file_text(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))
}

// ── Write text to file ────────────────────────────────────────────────────────
pub fn write_file_text(path: &Path, content: &str) -> Result<()> {
    let mut file = fs::File::create(path)
        .with_context(|| format!("Failed to create file: {}", path.display()))?;
    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write to file: {}", path.display()))
}

// ── Create new empty file ─────────────────────────────────────────────────────
pub fn create_file(path: &Path) -> Result<()> {
    if path.exists() {
        anyhow::bail!("File already exists: {}", path.display());
    }
    fs::File::create(path)
        .with_context(|| format!("Failed to create file: {}", path.display()))?;
    Ok(())
}

// ── Create directory (recursive) ──────────────────────────────────────────────
pub fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory: {}", path.display()))
}

// ── Copy file ─────────────────────────────────────────────────────────────────
pub fn copy_file(src: &Path, dst: &Path) -> Result<u64> {
    if !src.exists() {
        anyhow::bail!("Source file not found: {}", src.display());
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)
        .with_context(|| format!("Failed to copy {} -> {}", src.display(), dst.display()))
}

// ── Copy directory recursively ────────────────────────────────────────────────
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize> {
    if !src.is_dir() {
        anyhow::bail!("Source is not a directory: {}", src.display());
    }
    fs::create_dir_all(dst)?;
    let mut count = 0;
    for entry in WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src)?;
        let target = dst.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
            count += 1;
        }
    }
    Ok(count)
}

// ── Move (rename) file or directory ──────────────────────────────────────────
pub fn move_item(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        anyhow::bail!("Source not found: {}", src.display());
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    // Try rename first (fast, same filesystem)
    if fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    // Fallback: copy then delete
    if src.is_dir() {
        copy_dir_recursive(src, dst)?;
        fs::remove_dir_all(src)?;
    } else {
        fs::copy(src, dst)?;
        fs::remove_file(src)?;
    }
    Ok(())
}

// ── Rename (same directory) ───────────────────────────────────────────────────
pub fn rename_item(path: &Path, new_name: &str) -> Result<PathBuf> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot rename root path"))?;
    let new_path = parent.join(new_name);
    if new_path.exists() {
        anyhow::bail!("Target already exists: {}", new_path.display());
    }
    fs::rename(path, &new_path)
        .with_context(|| format!("Failed to rename: {} -> {}", path.display(), new_path.display()))?;
    Ok(new_path)
}

// ── Delete file ───────────────────────────────────────────────────────────────
pub fn delete_file(path: &Path) -> Result<()> {
    fs::remove_file(path)
        .with_context(|| format!("Failed to delete file: {}", path.display()))
}

// ── Delete directory (recursive) ─────────────────────────────────────────────
pub fn delete_dir(path: &Path) -> Result<()> {
    fs::remove_dir_all(path)
        .with_context(|| format!("Failed to delete directory: {}", path.display()))
}

// ── Delete any path ───────────────────────────────────────────────────────────
pub fn delete_item(path: &Path) -> Result<()> {
    if path.is_dir() {
        delete_dir(path)
    } else {
        delete_file(path)
    }
}

// ── List directory contents (shallow) ────────────────────────────────────────
pub fn list_dir(path: &Path) -> Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    // Directories first, then files; alphabetical within each group
    entries.sort_by(|a, b| {
        let a_dir = a.is_dir();
        let b_dir = b.is_dir();
        match (a_dir, b_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
                .cmp(
                    &b.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase(),
                ),
        }
    });
    Ok(entries)
}

// ── Gather detailed file statistics ──────────────────────────────────────────
pub fn get_file_stats(path: &Path) -> Result<FileStats> {
    let meta = fs::metadata(path)
        .with_context(|| format!("Failed to get metadata: {}", path.display()))?;

    let size = meta.len();
    let modified: Option<DateTime<Local>> = meta.modified().ok().map(|t| t.into());
    let created: Option<DateTime<Local>> = meta.created().ok().map(|t| t.into());

    #[cfg(unix)]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        format!("{:o}", meta.permissions().mode() & 0o777)
    };
    #[cfg(not(unix))]
    let permissions = if meta.permissions().readonly() {
        "readonly".to_string()
    } else {
        "read-write".to_string()
    };

    let md5 = if path.is_file() && size < 100 * 1024 * 1024 {
        compute_md5(path).ok()
    } else {
        None
    };

    let line_count = if path.is_file() && size < 10 * 1024 * 1024 {
        count_lines(path).ok()
    } else {
        None
    };

    Ok(FileStats {
        path: path.to_path_buf(),
        size,
        modified,
        created,
        md5,
        permissions,
        line_count,
    })
}

// ── Compute MD5 hash of a file ────────────────────────────────────────────────
pub fn compute_md5(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let digest = md5::compute(&buffer);
    Ok(format!("{:x}", digest))
}

// ── Count lines in a text file ────────────────────────────────────────────────
pub fn count_lines(path: &Path) -> Result<usize> {
    let content = fs::read(path)?;
    Ok(content.iter().filter(|&&b| b == b'\n').count())
}

// ── Compute total size of a directory ────────────────────────────────────────
pub fn dir_total_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

// ── Batch rename with prefix/suffix/find-replace ─────────────────────────────
#[derive(Debug, Clone)]
pub struct BatchRenameOptions {
    pub prefix: String,
    pub suffix: String,
    pub find: String,
    pub replace: String,
    pub use_numbering: bool,
    pub start_number: usize,
    pub number_padding: usize,
}

pub fn batch_rename(
    paths: &[PathBuf],
    opts: &BatchRenameOptions,
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut results = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let parent = path.parent().unwrap_or(Path::new("."));

        let mut new_stem = stem.clone();

        // Find & replace
        if !opts.find.is_empty() {
            new_stem = new_stem.replace(&opts.find, &opts.replace);
        }

        // Numbering
        let number_str = if opts.use_numbering {
            format!(
                "{:0width$}",
                opts.start_number + idx,
                width = opts.number_padding
            )
        } else {
            String::new()
        };

        let new_name = format!(
            "{}{}{}{}{}",
            opts.prefix, number_str, new_stem, opts.suffix, ext
        );
        let new_path = parent.join(&new_name);
        results.push((path.clone(), new_path));
    }

    Ok(results)
}

pub fn apply_batch_rename(renames: &[(PathBuf, PathBuf)]) -> Vec<(PathBuf, PathBuf, Result<()>)> {
    renames
        .iter()
        .map(|(src, dst)| {
            let result = fs::rename(src, dst).map_err(|e| anyhow::anyhow!(e));
            (src.clone(), dst.clone(), result)
        })
        .collect()
}

// ── Search files by name pattern ──────────────────────────────────────────────
pub fn search_files(root: &Path, pattern: &str, max_results: usize) -> Vec<PathBuf> {
    let pattern_lower = pattern.to_lowercase();
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .to_lowercase()
                .contains(&pattern_lower)
        })
        .map(|e| e.path().to_path_buf())
        .take(max_results)
        .collect()
}

// ── Format bytes as human-readable ────────────────────────────────────────────
pub fn format_size(bytes: u64) -> String {
    use humansize::{format_size, DECIMAL};
    format_size(bytes, DECIMAL)
}

// ── Read file for preview (first N lines) ─────────────────────────────────────
/// Returns (lines, total_line_count, is_text).
/// Reads up to `max_lines` lines. If the file appears binary, returns is_text=false.
pub fn read_file_preview(path: &Path, max_lines: usize) -> (Vec<String>, usize, bool) {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return (vec!["[无法读取文件]".to_string()], 0, false),
    };

    // Detect binary: any null byte in the first 8KB → treat as binary
    let probe = &bytes[..bytes.len().min(8192)];
    if probe.contains(&0u8) {
        return (vec![], 0, false);
    }

    let text = String::from_utf8_lossy(&bytes).to_string();
    let total_lines = text.lines().count();
    let lines: Vec<String> = text.lines().take(max_lines).map(|l| l.to_string()).collect();
    (lines, total_lines, true)
}

// ── Get disk space for the filesystem containing `path` ───────────────────────
#[cfg(unix)]
pub fn get_disk_space(path: &Path) -> Option<(u64, u64)> {
    use std::ffi::CString;
    use std::mem;

    let cpath = CString::new(path.to_string_lossy().as_bytes()).ok()?;
    unsafe {
        let mut stat: libc::statvfs = mem::zeroed();
        if libc::statvfs(cpath.as_ptr(), &mut stat) == 0 {
            let total = stat.f_blocks * stat.f_frsize;
            let free  = stat.f_bavail * stat.f_frsize;
            Some((total, free))
        } else {
            None
        }
    }
}

#[cfg(not(unix))]
pub fn get_disk_space(_path: &Path) -> Option<(u64, u64)> {
    None
}
