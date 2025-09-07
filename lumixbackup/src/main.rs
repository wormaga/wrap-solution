use anyhow::Result;
use blake3;
use chrono::{DateTime, Local};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rustyline::DefaultEditor; // NEW: line editor (arrow keys, history, etc.)
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

const ALLOWED_EXTS: [&str; 3] = ["jpg", "mov", "rw2"];

const SKIP_DIRS: [&str; 5] = [
    ".Spotlight-V100",
    ".fseventsd",
    ".Trashes",
    ".DocumentRevisions-V100",
    ".TemporaryItems",
];



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to Lumix folder. If omitted, defaults to /Volumes/LUMIX (must exist)
    #[arg(value_name = "INPUT_FOLDER")]
    input_folder: Option<PathBuf>, // CHANGED: now optional

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Gap threshold in minutes (default: 120)
    #[arg(short, long, default_value_t = 120)]
    gap: i64,
}

#[derive(Clone, Debug)]
struct FileEntry {
    path: PathBuf,
    timestamp: DateTime<Local>,
}

#[derive(Debug)]
struct Photoshoot {
    files: Vec<FileEntry>,
}

impl Photoshoot {
    fn new() -> Self {
        Self { files: Vec::new() }
    }

    fn add(&mut self, file: FileEntry) {
        self.files.push(file);
    }

    fn min_timestamp(&self) -> DateTime<Local> {
        self.files
            .iter()
            .map(|f| f.timestamp)
            .min()
            .expect("Photoshoot should have at least 1 file")
    }

    fn max_timestamp(&self) -> DateTime<Local> {
        self.files
            .iter()
            .map(|f| f.timestamp)
            .max()
            .expect("Photoshoot should have at least 1 file")
    }

    fn count_by_ext(&self, ext: &str) -> usize {
        self.files
            .iter()
            .filter(|f| {
                f.path
                    .extension()
                    .map_or(false, |e| e.eq_ignore_ascii_case(ext))
            })
            .count()
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine input path
    let input_path: PathBuf = match cli.input_folder {
        Some(p) => p,
        None => {
            let default = PathBuf::from("/Volumes/LUMIX");
            if default.exists() {
                println!("No INPUT_FOLDER specified. Using default: {}", default.display());
                default
            } else {
                eprintln!(
                    "No INPUT_FOLDER specified and default '/Volumes/LUMIX' was not found.\n\n"
                );
                eprintln!(
                    "Please run again and provide an input folder, e.g.:\n  lumixbackup /path/to/DCIM_root\n"
                );
                std::process::exit(2);
            }
        }
    };

    let verbose = cli.verbose;
    let gap_threshold_secs = cli.gap * 60;

    if verbose {
        println!("input_path var: {}", input_path.display());
        println!("verbose var: {}", verbose);
        println!("gap var: {} in minutes", cli.gap);
        println!("gap_threshold_secs var: {}", gap_threshold_secs);
    }

    let dcim_path = find_dcim_folder(&input_path)?;
    println!("Found DCIM folder: {}", dcim_path.display());

    let all_files = collect_all_files(&dcim_path)?;

    let shoots = detect_photoshoots(&all_files, gap_threshold_secs)?;

    println!("\nDetected {} photoshoots:", shoots.len());
    for (i, shoot) in shoots.iter().enumerate() {
        let counts: Vec<String> = ALLOWED_EXTS
            .iter()
            .map(|ext| format!("{} {}", shoot.count_by_ext(ext), ext.to_uppercase()))
            .collect();

        println!(
            "\n[{}] {} -> {} | {} | total {} files",
            i + 1,
            shoot.min_timestamp(),
            shoot.max_timestamp(),
            counts.join(" | "),
            shoot.files.len()
        );
    }

    // Display shoots and ask user to select them (example: user enters "1 3" to select shoot 1 and 3)
    let input = read_line_with_editor("Select shoots to backup (space-separated indexes): ")?;
    let selected_indexes: Vec<usize> = input
        .split_whitespace()
        .filter_map(|s| s.parse::<usize>().ok())
        .collect();

    // Ask user where to backup
    let output_folder = get_output_folder()?;
    if !output_folder.exists() {
        fs::create_dir_all(&output_folder).expect("Failed to create output folder");
    }

    for &idx in &selected_indexes {
        if let Some(shoot) = shoots.get(idx - 1) {
            println!("Backing up shoot {}", idx);
            if let Err(e) = copy_shoot_files(shoot, &output_folder, idx - 1, verbose) {
                eprintln!("Error copying shoot {}: {}", idx, e);
            }
        } else {
            println!("Invalid shoot index: {}", idx);
        }
    }

    println!("Backup completed!");

    Ok(())
}

fn find_dcim_folder(input_folder: &Path) -> Result<PathBuf> {
    for entry in WalkDir::new(input_folder)
        .min_depth(1)
        .max_depth(2)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.contains(&name.as_ref())
        })
    {
        let entry = entry?;
        if entry.file_type().is_dir()
            && entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("DCIM")
        {
            return Ok(entry.into_path());
        }
    }
    anyhow::bail!("DCIM folder not found in {}", input_folder.display())
}

fn collect_all_files(dcim_path: &Path) -> Result<Vec<FileEntry>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dcim_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.contains(&name.as_ref())
        })
    {
        if entry.file_type().is_file() {
            let ext = entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ALLOWED_EXTS.contains(&ext.as_str()) {
                let timestamp = get_file_creation_time(entry.path())?;
                files.push(FileEntry {
                    path: entry.into_path(),
                    timestamp,
                });
            }
        }
    }

    Ok(files)
}

fn get_file_creation_time(path: &Path) -> Result<DateTime<Local>> {
    let metadata = fs::metadata(path)?;
    let time = metadata.created().or_else(|_| metadata.modified())?;
    Ok(DateTime::<Local>::from(time))
}

fn detect_photoshoots(
    all_files: &Vec<FileEntry>,
    gap_threshold_secs: i64,
) -> Result<Vec<Photoshoot>> {
    // Step 1: Extract files for time analysis
    let mut media_files: Vec<FileEntry> = all_files
        .iter()
        .filter(|f| {
            let ext = f
                .path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            ALLOWED_EXTS.contains(&ext.as_str())
        })
        .cloned()
        .collect();

    if media_files.is_empty() {
        return Ok(vec![]);
    }

    // Step 2: Sort by timestamp
    media_files.sort_by_key(|f| f.timestamp);

    // Step 3: Group by gap threshold
    let mut shoots = Vec::new();
    let mut current_shoot = Photoshoot::new();
    current_shoot.add(media_files[0].clone());

    for i in 1..media_files.len() {
        let prev = &media_files[i - 1];
        let curr = &media_files[i];

        let gap = curr.timestamp.timestamp() - prev.timestamp.timestamp();
        if gap > gap_threshold_secs {
            shoots.push(current_shoot);
            current_shoot = Photoshoot::new();
        }
        current_shoot.add(curr.clone());
    }
    shoots.push(current_shoot);

    Ok(shoots)
}

// Function to get output folder path from user or use default
fn get_output_folder() -> io::Result<PathBuf> {
    let input = read_line_with_editor(
        "\nEnter path to your SSD (or output folder) (leave empty for default './auto-backup'): ",
    )?;
    let input = input.trim();

    if input.is_empty() {
        Ok(PathBuf::from("./auto-backup"))
    } else {
        let unescaped = unescape_backslashes(input);
        let expanded = expand_tilde(&unescaped);
        Ok(PathBuf::from(expanded))
    }
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return path.replacen("~", &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}


/// Read a line using a line editor that supports arrow keys and basic editing.
fn read_line_with_editor(prompt: &str) -> io::Result<String> {
    let mut rl = DefaultEditor::new().map_err(to_io_err)?;
    match rl.readline(prompt) {
        Ok(line) => Ok(line),
        Err(err) => Err(to_io_err(err)),
    }
}

/// Convert errors from rustyline to std::io::Error
fn to_io_err<E: std::fmt::Display>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, format!("{}", e))
}

/// Very small unescape helper: treats a backslash as escaping the next character.
/// This lets users paste shell-escaped paths like "/Volumes/Ana\\ Home/\\*Work\\ in\\ progress".
fn unescape_backslashes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            } else {
                // trailing backslash -> keep it
                out.push('\\');
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn copy_shoot_files(
    shoot: &Photoshoot,
    base_output: &Path,
    shoot_index: usize,
    verbose: bool,
) -> io::Result<()> {
    let start_dt = shoot.min_timestamp();
    let folder_name = format!(
        "{}_project_{}",
        start_dt.format("%Y-%m-%d_%H_%M"),
        shoot_index + 1
    );
    let shoot_folder = base_output.join(folder_name);
    fs::create_dir_all(&shoot_folder)?;

    // Collect unique extensions from shoot.files
    let extensions: HashSet<String> = shoot
        .files
        .iter()
        .filter_map(|f| {
            f.path
                .extension()
                .map(|ext| ext.to_string_lossy().to_string())
        })
        .collect();

    for ext in &extensions {
        fs::create_dir_all(shoot_folder.join(ext))?;
    }

    let num_cores = num_cpus::get();

    if verbose {
        println!("num_cores var: {}", num_cores);
    }

    if num_cores <= 2 {
        for ext in extensions {
            let subfolder = shoot_folder.join(&ext);
            fs::create_dir_all(&subfolder)?; // folder must be already created, but keep this safe-guard
            copy_files_by_ext(&shoot.files, &ext, &subfolder, verbose)?;
        }
    } else {
        // Prepare progress bar
        let total_files = shoot.files.len();
        let pb = if verbose {
            None
        } else {
            let pb = ProgressBar::new(total_files as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.yellow} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files copied")
                    .unwrap()
                    .progress_chars("#>-")
            );
            Some(Arc::new(Mutex::new(pb)))
        };

        let pb_clone = pb.clone();

        // Parallel copy
        shoot.files.par_iter().for_each(|file| {
            let ext = file
                .path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let dest_path = shoot_folder.join(&ext).join(file.path.file_name().unwrap());
            if let Err(e) = fs::copy(&file.path, &dest_path) {
                eprintln!("Failed to copy {:?} -> {:?}: {}", file.path, dest_path, e);
            }

            // Verify integrity
            match (file_checksum(&file.path), file_checksum(&dest_path)) {
                (Ok(src_hash), Ok(dest_hash)) => {
                    if src_hash != dest_hash {
                        eprintln!("WARNING: Integrity check failed for {:?}", file.path);
                    } else if verbose {
                        println!("Verified {:?}", file.path);
                    }
                }
                (Err(e), _) | (_, Err(e)) => {
                    eprintln!("Checksum error for {:?}: {}", file.path, e);
                }
            }

            if verbose {
                println!("Copied {:?} -> {:?}", file.path, dest_path);
            } else if let Some(ref pb) = pb_clone {
                pb.lock().unwrap().inc(1);
            }
        });

        // Finish progress bar
        if let Some(pb) = pb {
            pb.lock()
                .unwrap()
                .finish_with_message(format!("{} files copied", total_files));
        }
    }

    Ok(())
}

fn copy_files_by_ext(
    files: &[FileEntry],
    ext: &str,
    dest_folder: &Path,
    verbose: bool,
) -> std::io::Result<()> {
    let filtered: Vec<&FileEntry> = files
        .iter()
        .filter(|f| {
            f.path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case(ext))
                .unwrap_or(false)
        })
        .collect();

    if filtered.is_empty() {
        return Ok(());
    }

    let pb = if verbose {
        None
    } else {
        let pb = ProgressBar::new(filtered.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-")
        );
        pb.set_message(format!("Copying {} files", ext.to_uppercase()));
        Some(pb)
    };

    let total_file_count = filtered.len();
    for file in filtered {
        let dest_path = dest_folder.join(file.path.file_name().unwrap());
        fs::copy(&file.path, &dest_path)?;

        // Verify integrity
        match (file_checksum(&file.path), file_checksum(&dest_path)) {
            (Ok(src_hash), Ok(dest_hash)) => {
                if src_hash != dest_hash {
                    eprintln!("WARNING: Integrity check failed for {:?}", file.path);
                } else if verbose {
                    println!("Verified {:?}", file.path);
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                eprintln!("Checksum error for {:?}: {}", file.path, e);
            }
        }

        if verbose {
            println!("Copied {:?} to {:?}", file.path, dest_path);
        } else if let Some(ref pb) = pb {
            pb.inc(1);
        }
    }

    if let Some(pb) = pb {
        pb.finish_with_message(format!(
            "{} {} files copied",
            total_file_count,
            ext.to_uppercase()
        ));
    }

    Ok(())
}

fn file_checksum(path: &Path) -> io::Result<blake3::Hash> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize())
}
