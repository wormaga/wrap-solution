use reqwest::{Client, Error};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process;
use std::process::Command;
extern crate reqwest;

static BIN: &str = "bin"; //~/bin
static CLI_PROJECTS: &str = "cli-projects"; //~/cli-projects

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    last_update: String,
    tools: Vec<Tool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    name: String,
    version: String,
    files: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    location: String,
    filename: String,
    url: String,
}

impl Tool {
    pub fn is_installed(&self) -> bool {
        Command::new(&self.name)
        .arg("--version")
        .output()
        .is_ok()
    }

    pub fn is_update_available(&self) -> bool {
        if !self.is_installed() {
            return true;
        }

        let output = Command::new(&self.name)
        .arg("--version")
        .output()
        .unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();

        let installed_version = Version::parse(stdout.trim()).unwrap();
        let latest_version = Version::parse(&self.version).unwrap();

        if installed_version > latest_version {
            println!("Something is wrong, installed {}, the latest available is {}.", installed_version, latest_version);
        }

        installed_version < latest_version
    }

    pub fn install_description(&self) -> String {
        if !self.is_installed() {
            return "".to_string();
        }

        let output = Command::new(&self.name)
            .arg("--version")
            .output()
            .unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();

        let installed_version = Version::parse(stdout.trim()).unwrap();
        let latest_version = Version::parse(&self.version).unwrap();

        if installed_version < latest_version {
            return format!("(update available: {}-->{})", installed_version, latest_version);
        }

        if installed_version == latest_version {
            return format!("(Latest installed {})", latest_version);
        }

        // if installed_version > latest_version {
        //     return format!("(Latest installed {})", latest_version);
        // }

        return "".to_string();
    }
}

impl Product {
    pub fn filter_tools_by_user(&self) -> Vec<&Tool> {
        // Print the list of tools with their indices
        println!("Select one or more programs by their number (separated by space):");
        for (i, tool) in self.tools.iter().enumerate() {
            println!("{}) {} {}", i + 1, tool.name, tool.install_description());
        }

        let mut selected_indices: Vec<usize>;
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            selected_indices = input
                .trim()
                .split(' ')
                .map(|x| x.parse::<usize>().unwrap_or(usize::MAX))
                .filter(|&x| x > 0 && x - 1 < self.tools.len())
                .collect();
            if !selected_indices.is_empty() {
                break;
            }
            println!("Invalid input. Try again:");
        }
        // println!("selected_indices {:#?}", selected_indices); //debug

        // Filter tools by index
        let filtered_tools = self
            .tools
            .iter()
            .enumerate()
            .filter(|(i, _)| selected_indices.contains(&(i + 1)))
            .map(|(_, tool)| tool)
            .collect::<Vec<_>>();

        filtered_tools
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let product: Product = Client::new()
        .get("https://raw.githubusercontent.com/wormaga/wrap-solution/main/wrap.json")
        .send()
        .await?
        .json()
        .await?;
    //println!("{:#?}", product); //debug

    let selected_tools = product.filter_tools_by_user();

    if selected_tools.is_empty() {
        println!("No tools were selected. Exiting the program.");
        process::exit(0);
    }

    for tool in selected_tools {
        if tool.is_update_available() {
            //println!("Debug: update is available"); //debug
            install_tool(&tool).await
        } else {
            println!("The latest version is installed.");
        }
    }

    Ok(())
}

async fn install_tool(tool: &Tool) {
    //println!("{:#?}", tool); //debug

    ensure_rust_up_to_date();

    let project_name = &tool.name;

    let home_dir = dirs::home_dir().expect("failed to get home directory");
    let bin_dir = home_dir.join(BIN);
    let cli_projects_dir = home_dir.join(CLI_PROJECTS);

    set_current_directory(&bin_dir);
    set_current_directory(&cli_projects_dir);

    //deleting existing project code
    let tool_dir = cli_projects_dir.join(&project_name);
    if tool_dir.exists() {
        delete_folder(&tool_dir).expect("failed to delete litegallery directory");
        println!("Deleted existing {} project folder.", &project_name);
    }

    // create a new rust project
    println!("Creating a new rust project");
    let output = Command::new("cargo")
        .arg("new")
        .arg(&project_name)
        .output()
        .expect("failed create a new cargo project.");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    //go inside the project
    set_current_directory(&tool_dir);

    println!("Downloading up to date files from github");
    for asset in &tool.files {
        set_current_directory(&(tool_dir.join(&asset.location)));

        download_file(&asset.url, &asset.filename)
            .await
            .expect("failed to download file.");
    }
    println!("All files downlaoded.");

    //go inside the project
    set_current_directory(&tool_dir);

    let output = Command::new("pwd")
        .output()
        .expect("failed compile a project.");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("Compiling program {}", &project_name);
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .output()
        .expect("failed compile a project.");

    println!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("Cargo build failed: {}", error_message);
        std::process::exit(1);
    }

    //move compiled program to ${HOME}/bin folder
    println!("Coping program {} to ~/bin folder", &project_name);
    let output = Command::new("cp")
        .arg(&format!("target/release/{}", &project_name))
        .arg(&bin_dir)
        .output()
        .expect("failed compile a project.");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("{} is installed.", &project_name);
}

fn set_current_directory(dir: &Path) {
    // Create the folder if it does not exist
    if !dir.exists() {
        fs::create_dir(&dir).expect("failed to create directory");
    }

    // Change the current directory
    env::set_current_dir(&dir).expect("failed to change directory");
}

fn delete_folder(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Delete the folder
    fs::remove_dir_all(dir)?;

    Ok(())
}

async fn download_file(url: &str, filename: &str) -> Result<(), Error> {
    let resp = reqwest::get(url).await.expect("request failed");
    assert!(resp.status().is_success());
    let body = resp.text().await.expect("body invalid");
    let mut out = File::create(filename).expect("failed to create file");
    io::copy(&mut body.as_bytes(), &mut out).expect("failed to copy content");

    Ok(())
}

fn ensure_rust_up_to_date() {
    use std::process::Command;

    // Check rustc version
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("Failed to run rustc");

    let version_str = String::from_utf8_lossy(&output.stdout);
    println!("Current Rust version: {}", version_str.trim());

    // Optionally parse the version number and compare with a minimum
    // For simplicity, just update Rust every time
    println!("Updating Rust toolchain...");
    let status = Command::new("rustup")
        .arg("update")
        .arg("stable")
        .status()
        .expect("Failed to update Rust via rustup");

    if !status.success() {
        eprintln!("Rust update failed. Please update manually.");
        std::process::exit(1);
    }

    println!("Rust is up to date!");
}
