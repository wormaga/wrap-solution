### **Purpose**

It scans a given folder (usually from a Lumix camera), finds the `DCIM` folder, detects “photoshoots” based on time gaps between files, and copies selected shoots to an output directory, grouping them by file extension.

---

### **Main Behavior**

1. **Parse command line arguments** (`input_folder`, `verbose`, and a `gap` in minutes to detect shoot boundaries).
2. **Find `DCIM` folder** inside the provided input path (up to 2 subdirectory levels deep).
3. **Collect all files** with extensions `.jpg`, `.mov`, `.rw2` from that folder.
4. **Read file timestamps** (creation date, fallback to modification date).
5. **Group files into “photoshoots”** if the gap between consecutive files exceeds the given threshold.
6. **Display detected shoots** and file counts per extension.
7. **Ask the user** which shoots to back up (interactive prompt).
8. **Ask the user** for an output folder (default: `./auto-backup`).
9. **Copy files** from each selected shoot into subfolders organized by date/time and file extension.

   * Uses multi-threaded copying with a progress bar if the system has more than 2 CPU cores.

---

### **What You Should Be Aware Of**

* **It’s interactive** — needs user input for selecting shoots and backup location.
* **It doesn’t handle overwrites carefully** — if the destination already has files with the same name, they’ll be silently overwritten.
* **Timestamps come from file system metadata**, not EXIF, so if your camera clock was wrong or metadata was altered, grouping might be inaccurate.

---