use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use std::io;
use std::process;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::fs::File;
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


#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let product: Product = Client::new()
        .get("https://raw.githubusercontent.com/wormaga/wrap-solution/main/wrap.json")
        .send()
        .await?
        .json()
        .await?;
    //println!("{:#?}", product); //debug


    // Extract the tool names into an array
    let tool_names: Vec<&str> = product.tools.iter().map(|t| t.name.as_str()).collect();
    //println!("{:?}", tool_names);//debug

    let selected_strings = get_selected_strings(&tool_names);

    // for string in selected_strings {
    //     println!("{}", string); //debug
    // }

    if selected_strings.is_empty() {
        println!("No tools were selected. Exiting the program.");
        process::exit(0);
    }


    for cli in selected_strings {
        let found_tool =  find_tool_by_name(&product.tools, &cli);
        //TODO check if tool is installed
        //TODO check if tool has the lates version
        install_tool(found_tool).await
    }

    Ok(())
}


async fn install_tool(found_tool: Option<&Tool>) {
    //println!("{:#?}", found_tool); //debug

    if let Some(found_tool) = found_tool {
        let project_name = &found_tool.name;

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
        for asset in &found_tool.files {
            set_current_directory(&(tool_dir.join(&asset.location)));


            download_file(&asset.url, &asset.filename).await.expect("failed to download file.");
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
        .arg(&format!("target/release/{}",&project_name))
        .arg(&bin_dir)
        .output()
        .expect("failed compile a project.");
        println!("{}", String::from_utf8_lossy(&output.stdout));

        println!("Program finished.");

    } else {
        println!("Tool not found.");
    }
}

fn find_tool_by_name<'a>(tools: &'a Vec<Tool>, name: &str) -> Option<&'a Tool> {
    for tool in tools {
        if tool.name == name {
            return Some(tool);
        }
    }
    None
}


fn get_selected_strings<'a>(strings: &[&'a str]) -> Vec<&'a str> {
    println!("Select one or more programs by their number (separated by space):");
    for (index, string) in strings.iter().enumerate() {
        println!("{}) {}", index+1, string);
    }

    let mut selected_indices: Vec<usize>;
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        selected_indices = input
            .trim()
            .split(' ')
            .map(|x| x.parse::<usize>().unwrap_or(usize::MAX))
            .filter(|&x| x > 0 && x-1 < strings.len())
            .collect();
        if !selected_indices.is_empty() {
            break;
        }
        println!("Invalid input. Try again:");
    }

    selected_indices.iter().map(|&i| strings[i-1]).collect()
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