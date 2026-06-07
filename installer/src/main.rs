use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const PAYLOAD: &[u8] = include_bytes!("../../venus_compiler/target/release/vscript.exe");

fn main() {
    println!("========================================");
    println!("  VenusScript Compiler v0.2.0 Installer");
    println!("========================================");
    println!();

    let local_appdata = match env::var("LOCALAPPDATA") {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Error: LOCALAPPDATA environment variable not found.");
            pause_and_exit(1);
        }
    };

    let install_dir = PathBuf::from(local_appdata).join("VenusScript").join("bin");

    if !install_dir.exists() {
        println!("Creating installation directory at {:?}", install_dir);
        if let Err(e) = fs::create_dir_all(&install_dir) {
            eprintln!("Failed to create directory: {}", e);
            pause_and_exit(1);
        }
    }

    let exe_path = install_dir.join("vscript.exe");
    println!("Installing vscript.exe...");
    
    if let Err(e) = fs::write(&exe_path, PAYLOAD) {
        eprintln!("Failed to write vscript.exe: {}", e);
        pause_and_exit(1);
    }
    println!("Successfully copied vscript.exe.");

    println!("Adding VenusScript to the User PATH...");

    // We use PowerShell to cleanly add to the user's PATH environment variable
    let ps_script = format!(
        r#"
        $installDir = "{}"
        $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($userPath -notlike "*$installDir*") {{
            $newPath = "$userPath;$installDir"
            [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
            Write-Host "Added to PATH."
        }} else {{
            Write-Host "Already in PATH."
        }}
        "#,
        install_dir.display()
    );

    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&ps_script)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                println!("{}", String::from_utf8_lossy(&out.stdout).trim());
            } else {
                eprintln!("Warning: Failed to update PATH. {}", String::from_utf8_lossy(&out.stderr));
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not execute PowerShell to update PATH: {}", e);
        }
    }

    println!();
    println!("Installation Complete! You can now use the 'vscript' command in any new terminal.");
    println!("Press Enter to exit...");
    
    // Pause before closing the terminal window (so double clickers can read)
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}

fn pause_and_exit(code: i32) -> ! {
    println!("Press Enter to exit...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    std::process::exit(code);
}
