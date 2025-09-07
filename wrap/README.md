# Rust CLI Tool Installer

A Rust-based CLI tool manager that fetches Rust project metadata from GitHub, checks versions, downloads source files, builds the projects, and installs binaries in `~/bin`.

---

## Features

- Fetch a list of Rust CLI tools from a remote JSON file.
- Check if tools are installed and whether updates are available.
- Select multiple tools to install/update via interactive prompt.
- Download source files from GitHub.
- Compile projects using Cargo.
- Copy binaries to `~/bin` for easy execution.

---

## Diagram

          ┌───────────────────────────┐
          │ Start Program (main)      │
          └─────────────┬─────────────┘
                        │
                        ▼
         ┌─────────────────────────────┐
         │ Fetch JSON metadata from    │
         │ GitHub (Product & Tools)    │
         └──────────────┬──────────────┘
                        │
                        ▼
         ┌─────────────────────────────┐
         │ Display tools & versions    │
         │ to user for selection       │
         └──────────────┬──────────────┘
                        │
                        ▼
         ┌─────────────────────────────┐
         │ User selects tools          │
         └──────────────┬──────────────┘
                        │
                        ▼
         ┌─────────────────────────────┐
         │ For each selected tool:     │
         └──────────────┬──────────────┘
                        │
                        ▼
         ┌─────────────────────────────┐
         │ Check if installed &        │
         │ update available            │
         └──────────────┬──────────────┘
                        │
      ┌─────────────────┴───────────────────┐
      │                                     │
      ▼                                     ▼
Not installed / update available     Latest installed
      │                                     │
      ▼                                     │
 ┌───────────────┐                          │
 │ Install tool  │                          │
 └─────┬─────────┘                          │
       │                                    │  
       ▼                                    │
┌────────────────┐                          │
│ Delete old     │                          │
│ project folder │                          │
└──────┬─────────┘                          │
       │                                    │
       ▼                                    │
┌────────────────┐                          │
│ Create new     │                          │
│ cargo project  │                          │
└──────┬─────────┘                          │
       │                                    │
       ▼                                    │
┌────────────────┐                          │
│ Download files │                          │
│ from GitHub    │                          │
└──────┬─────────┘                          │
       │                                    │
       ▼                                    │
┌────────────────┐                          │
│ Build project  │                          │
│ with Cargo     │                          │
└──────┬─────────┘                          │
       │                                    │
       ▼                                    │
┌────────────────┐                          │
│ Copy binary    │                          │
│ to ~/bin       │                          │
└──────┬─────────┘                          │
       │                                    │
       ▼                                    ▼
 ┌───────────────┐                 ┌───────────────────┐
 │ Installation  │                 │ Skip installation │
 │ complete      │                 │ (already latest)  │
 └───────────────┘                 └───────────────────┘
       │
       ▼
 ┌───────────────┐
 │ End Program   │
 └───────────────┘
