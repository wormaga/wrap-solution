# Lumix Backup Tool

A CLI utility written in Rust for **analyzing and backing up Lumix (and other camera) SD cards**.  
It automatically detects shoots based on time gaps in photos, organizes them into timestamped project folders, and copies files into type-based subfolders (`jpg/`, `rw2/`, `mov/`, etc.).

---

## **Purpose**

It scans a given folder (usually a Lumix SD card), finds the `DCIM` folder, detects “photoshoots” based on time gaps between files, and copies selected shoots to an output directory, grouping them by file extension.

---

## **Main Behavior**

1. **Parse command line arguments** (`input_folder`, `verbose`, and `gap` in minutes to detect shoot boundaries).  
   - `input_folder` is optional; defaults to `/Volumes/LUMIX` if omitted.
2. **Find `DCIM` folder** inside the provided input path (up to 2 subdirectory levels deep).  
   - Skips system directories (`.Spotlight-V100`, `.fseventsd`, `.Trashes`, `.DocumentRevisions-V100`, `.TemporaryItems`) and hidden files (dotfiles).
3. **Collect all files** with extensions `.jpg`, `.mov`, `.rw2` from that folder.
4. **Read file timestamps** (creation date, fallback to modification date).
5. **Group files into “photoshoots”** if the gap between consecutive files exceeds the given threshold.
6. **Display detected shoots** with file counts per extension.
7. **Detect duplicate filenames** inside each shoot:
   - If duplicates are found, the program prints a warning and **stops** before copying.
8. **Ask the user** which shoots to back up (interactive prompt with arrow keys and history support).  
   - Escaped spaces and `~` expansion are supported for paths.
9. **Ask the user** for an output folder (default: `./auto-backup`).
10. **Copy files** from each selected shoot into subfolders organized by date/time and file extension.  
    - Uses multi-threaded copying if the system has more than 2 CPU cores.
    - Verifies integrity via Blake3 checksums.

---

## **Usage Examples**

### Basic run (default input folder `/Volumes/LUMIX`):

```bash
$ lumixbackup
No INPUT_FOLDER specified. Using default: /Volumes/LUMIX
Found DCIM folder at: /Volumes/LUMIX/DCIM
Detected 2 photoshoots:
[1] 2025-09-03 14:32:00 -> 2025-09-03 15:10:00 | 50 JPG | 50 RW2 | 10 MOV | total 110 files
[2] 2025-09-05 09:10:00 -> 2025-09-05 10:05:00 | 80 JPG | 80 RW2 | 5 MOV | total 165 files

Select shoots to backup (space-separated indexes): 1 2

Enter output folder path [default: ./auto-backup]: ~/Pictures/Backups
```