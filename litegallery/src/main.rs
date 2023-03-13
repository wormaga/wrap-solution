use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process;
use std::process::{Command, Stdio};
use colored::Colorize;
use spinners::{Spinner, Spinners};

fn set_current_dir() -> bool { 
    // Get the current working directory 
    let current_dir = env::current_dir().unwrap(); 
    println!("The current directory for search is: {}", 
        format!("{}", current_dir.to_str().unwrap())
        .green()); 
 
    // Prompt the user if they want to modify the path 
    println!("{}", format!("Do you want to modify it? (y/N)").yellow()); 
 
    let mut input = String::new(); 
    io::stdin().read_line(&mut input).unwrap(); 
 
    if input.trim() == "y" || input.trim() == "Y"{ 
        println!("{}", format!("Enter the new path:").yellow()); 
 
        let mut new_path = String::new(); 
        io::stdin().read_line(&mut new_path).unwrap(); 
 
        // Set the new working directory 
        match env::set_current_dir(new_path.replace("\\","").trim()) { 
            Ok(_) => { 
                println!("The search directory is now: {}", 
                format!("{}", env::current_dir().unwrap().to_str().unwrap())
                .green()); 
                return true; 
            }, 
            Err(error) => { 
                println!("Error setting new working directory: {}", error); 
                println!("{}", format!("The new path for the working directory is incorrect. Please try again.").red()); 
                return false; 
            } 
        } 
    }

    println!("No changes made, search will perform in: {}", 
        format!("{}", env::current_dir().unwrap().to_str().unwrap())
        .green()); 
 
    return true; 
} 

fn open_directory(directory: &str) {
    // Use the `xdg-open` command on Linux and macOS
    if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(directory)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to execute command");
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open")
            .arg(directory)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to execute command");
    // Use the `explorer.exe` command on Windows
    } else if cfg!(target_os = "windows") {
        Command::new("explorer.exe")
            .arg(directory)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to execute command");
    } else {
        panic!("Unsupported operating system");
    };
}

fn should_continue() -> bool {
    println!("{}", format!("Do you want to continue? (y/N)").yellow());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim();
    if input == "y" || input == "Y" {
        true
    } else {
        false
    }
}

fn copy_files(filenames: Vec<String>, output_dir: &str) {
    println!("Start copied files, it will take some time...");

    // Create the output directory if it doesn't exist
    if !Path::new(&output_dir).exists() {
        fs::create_dir(&output_dir).expect("Failed to create output directory");
    }

    // Create and start a spinner
    let mut sp = Spinner::new(Spinners::Line, "Coping files".into());
    
    for filename in filenames {
        let file_path = Path::new(&filename);

        // Check if the file is a regular file (not a directory)
        if file_path.is_file() {
            let dest_path = Path::new(&output_dir).join(file_path.file_name().unwrap());
            match fs::copy(&file_path, &dest_path) {
                Ok(_) => {},
                Err(e) => println!("Failed to copy file {}: {}", format!("{}", filename).red(), e),
            }
        }
    }

    //stop spinner
    sp.stop();
    println!("\n{}", format!("{}", "Finished copied files").green());
}

fn find_files(filenames: Vec<String>) -> Vec<String> {
    let mut paths = Vec::new();
    let mut not_found = Vec::new();

    for filename in filenames {
        // Check if the file is in the current directory
        let file_path = Path::new(&filename);
        if file_path.exists() {
            paths.push(file_path.to_string_lossy().to_string());
            continue;
        }

        // If the file is not in the current directory, look for it in subfolders
        let current_dir = env::current_dir().unwrap();
        let mut found = false;
        for entry in fs::read_dir(current_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let file_path = path.join(&filename);
                if file_path.exists() {
                    paths.push(file_path.to_string_lossy().to_string());
                    found = true;
                    break;
                }
            }
        }

        // If the file was not found, add it to the list of not found files
        if !found {
            not_found.push(filename);
        }
    }

    println!("Found Files:  {:?}", paths);

    // Print the list of not found files
    if !not_found.is_empty() {
        println!("The following files were ({}):", format!("not found {}", not_found.len()).red());
        for filename in not_found {
            print!("{},", format!("{}", filename).red());
        }
        println!("");
    }
    paths
}


fn generate_filenames(filenames: Vec<String>, extensions: &Vec<String>) -> Vec<String> {
    let mut generated_filenames = vec![];
    for filename in filenames {
        if filename.is_empty() {
            continue;
        }

        let path = Path::new(&filename);
        let stem = path.file_stem().unwrap().to_str().unwrap();
        for extension in extensions {
            let new_filename = format!("{}.{}", stem, extension);
            generated_filenames.push(new_filename);
        }
    }
    generated_filenames
}

fn get_filenames_and_extensions() -> (Vec<String>, Vec<String>) {
    // Prompt the user for a string of comma-separated filenames
    println!("{}", format!("Enter a list of filenames (separated by commas or spaces):").yellow());
    let mut filenames_input = String::new();
    io::stdin().read_line(&mut filenames_input).unwrap();
    let mut filenames: Vec<&str> = filenames_input.split(',').map(|s| s.trim()).collect();
  
    // If the list of filenames has less than two items, try to split the input string by spaces
    if filenames.len() < 2 {
        filenames = filenames_input.split(' ').map(|s| s.trim()).collect();
    }

    // Print a warning message if the list of filenames has less than two items
    if filenames.len() < 2 {
        println!("Warning: The list of filenames has less than two items!");
    }

    // Prompt the user for a string of comma-separated extensions
    println!("{}", format!("Enter a list of extensions (separated by commas):").yellow());
    let mut extensions_input = String::new();
    io::stdin().read_line(&mut extensions_input).unwrap();
    let extensions: Vec<&str> = extensions_input.split(',').map(|s| s.trim()).collect();

    // Print a warning message if the list of extensions has less than one item
    if extensions.len() < 1 {
        println!("Warning: The list of extensions has less than one item!");
    }

    // Convert the filenames and extensions from &str to String
    let filenames: Vec<String> = filenames.iter().map(|s| s.to_string()).collect();
    let extensions: Vec<String> = extensions.iter().map(|s| s.to_string()).collect();

    // Return the filenames and extensions as a tuple
    (filenames, extensions)
}

fn main() {
    // displaying version when "-v" or "--version" is passed
    let args: Vec<String> = env::args().collect();
    let first_passed_argument = &args[1];
    //print!("Inputs: {}", &first_passed_argument);

    let version_parameter: String = String::from("--version");
    let version_parameter_short: String = String::from("-v");
    if first_passed_argument.eq(&version_parameter) || first_passed_argument.eq(&version_parameter_short) {
        print!("{}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }


    // Start of the program
    println!("Version: {}\n", env!("CARGO_PKG_VERSION"));

    let mut success = false; 
    while !success { 
        success = set_current_dir(); 
    } 

    let (filenames, extensions) = get_filenames_and_extensions();
	//println!("Filenames: {:?}", filenames);
	//println!("##############");
    //println!("Extensions: {:?}", extensions);
    //println!("##############");
    let generated_filenames = generate_filenames(filenames, &extensions);
	println!("Generated filenames: {:?}", generated_filenames);
    let paths = find_files(generated_filenames);
    println!("{}", format!("Found {} files.", paths.len()).green());
	
	if !should_continue() {
        println!("Exiting...\n\n");
        process::exit(0);
    }
	

    // Prompt the user for the output directory
    println!("{}", format!("Enter the output directory:").yellow());
    let mut output_dir = String::new();
    io::stdin().read_line(&mut output_dir).unwrap();
    let output_dir = output_dir.trim();
    println!("The output directory is: \n    {}", format!("{}", output_dir).green());

    copy_files(paths, output_dir);
	
    open_directory(output_dir);

    println!("Progrma finished, you can now close this window.\n\n");
}
