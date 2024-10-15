use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn list_printers(base_name: &str, additional_names: &[&str]) -> Vec<String> {
    let output = Command::new("lpstat")
        .arg("-p")
        .output()
        .expect("Failed to execute lpstat");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut printers = Vec::new();

    // Collect printers matching the base name (with suffixes)
    for line in stdout.lines() {
        if line.contains(base_name) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                printers.push(parts[1].to_string());
            }
        }
    }

    // Collect additional specific printers
    for &name in additional_names {
        if stdout.contains(name) {
            let parts: Vec<&str> = stdout.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == name {
                    printers.push(part.to_string());
                    break;
                }
            }
        }
    }

    printers
}

fn get_job_id(printer: &str) -> Option<String> {
    let output = Command::new("lpstat")
        .arg("-o")
        .output()
        .expect("Failed to execute lpstat");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().find_map(|line| {
        if line.contains(printer) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                Some(parts[0].to_string())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn check_job_status(job_id: &str) -> bool {
    let output = Command::new("lpstat")
        .arg("-W")
        .arg("completed")
        .output()
        .expect("Failed to execute lpstat");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(job_id)
}

fn print_and_manage_files(printer: &str, downloads_folder: &str) {
    let zpl_files: Vec<_> = fs::read_dir(downloads_folder)
        .expect("Failed to read directory")
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext.to_string_lossy().to_lowercase() == "zpl")
                .unwrap_or(false)
        })
        .collect();

    if zpl_files.is_empty() {
        println!("No .zpl files found in {}", downloads_folder);
        return;
    }

    if zpl_files.len() > 1 {
        println!(
            "Multiple .zpl files found. Do you want to mark them as used after printing? (y/n)"
        );
        let mut confirmation = String::new();
        io::stdin()
            .read_line(&mut confirmation)
            .expect("Failed to read line");
        if confirmation.trim().to_lowercase() != "y" {
            println!("Files will not be marked as used. Exiting.");
            return;
        }
    }

    for entry in &zpl_files {
        let file_path = entry.path();
        if file_path.is_file() {
            println!("Attempting to print {:?}", file_path.display());

            let status = Command::new("lpr")
                .arg("-P")
                .arg(printer)
                .arg("-o")
                .arg("raw")
                .arg(file_path.display().to_string())
                .stdin(Stdio::null())
                .status()
                .expect("Failed to execute lpr");

            if !status.success() {
                println!("Failed to start print job on {}", printer);
                continue;
            }

            // Retrieve the job ID
            let job_id = match get_job_id(printer) {
                Some(id) => {
                    println!("Print job started with ID: {}", id);

                    let start_time = Instant::now();
                    let timeout = Duration::new(60, 0); // Increase timeout if necessary
                    let mut job_completed = false;

                    while start_time.elapsed() < timeout {
                        if check_job_status(&id) {
                            job_completed = true;
                            break;
                        }
                        sleep(Duration::new(1, 0));
                    }

                    if job_completed {
                        println!(
                            "Print job completed. Marking {:?} as used.",
                            file_path.display()
                        );
                        let new_file_path = file_path.with_extension("used");
                        fs::rename(&file_path, new_file_path).expect("Failed to mark file as used");
                    } else {
                        println!("Print job did not complete within the timeout period.");
                        // Optionally cancel the job if necessary
                        Command::new("cancel")
                            .arg(&id)
                            .status()
                            .expect("Failed to cancel job");
                    }
                    id
                }
                None => {
                    println!("Could not find the job ID for the print job.");
                    continue;
                }
            };
        }
    }
    if zpl_files.len() > 1 {
        println!("Please re-download the .zpl files.");
    }
}

fn main() {
    let base_name = "ZTC-ZP-450-200dpi";
    let additional_names = ["zprint", "Zebra-ZP-450"];
    let downloads_folder = format!("{}/Downloads", std::env::var("HOME").unwrap());

    let printers = list_printers(base_name, &additional_names);

    if printers.is_empty() {
        println!(
            "No printers found with base name '{}' or specified names.",
            base_name
        );
        return;
    }

    // Display the list of available printers
    println!("Available printers:");
    for (index, printer) in printers.iter().enumerate() {
        println!("{}: {}", index + 1, printer);
    }

    // Prompt the user to select a printer
    let mut choice = String::new();
    print!("Enter the number of the printer you want to use: ");
    io::stdout().flush().expect("Failed to flush stdout");
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");
    let choice: usize = match choice.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid input. Exiting.");
            return;
        }
    };

    if choice == 0 || choice > printers.len() {
        println!("Invalid printer number. Exiting.");
        return;
    }

    let selected_printer = &printers[choice - 1];
    println!("Selected printer: {}", selected_printer);

    print_and_manage_files(selected_printer, &downloads_folder);
    println!("Script completed.");
}
