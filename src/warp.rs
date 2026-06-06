use crate::AppError;
use std::env;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::process::Command as StdCommand;
use tokio::process::Command;

/// Helper function to detect available terminal emulators on Linux.
/// Returns the terminal name and the appropriate flag to execute a script.
fn find_terminal() -> Option<(String, Vec<String>)> {
    let terminals = vec![
        ("gnome-terminal", vec!["--".to_string()]),
        ("ptyxis", vec!["--".to_string()]),
        ("kgx", vec!["-e".to_string()]),
        ("konsole", vec!["-e".to_string()]),
        ("xfce4-terminal", vec!["-e".to_string()]),
        ("mate-terminal", vec!["-e".to_string()]),
        ("lxterminal", vec!["-e".to_string()]),
        ("xterm", vec!["-e".to_string()]),
    ];

    for (term, args) in terminals {
        if StdCommand::new("which")
            .arg(term)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some((term.to_string(), args));
        }
    }
    None
}

/// Creates an interactive terminal bash script to guide the user through installing Cloudflare WARP,
/// then opens it in an available terminal emulator on Linux (Fedora/Ubuntu/Debian).
pub async fn install_warp() -> Result<String, AppError> {
    println!("[WARP Installer] Starting Cloudflare WARP interactive installer process...");

    let script_content = include_str!("install_warp_wizard.sh");

    // Use temp_dir safely to write our shell script with a unique name using PID
    let temp_dir = env::temp_dir();
    let pid = std::process::id();
    let script_name = format!("install_warp_wizard_{}.sh", pid);
    let script_path = temp_dir.join(&script_name);
    let script_path_str = script_path.to_str().ok_or_else(|| {
        AppError::WarpInstaller("Failed to construct script path string".to_string())
    })?;

    // Create the interactive script file securely with atomic owner-only permissions (0700)
    // and fail if the file/symlink already exists to prevent symlink attacks.
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o700)
        .open(&script_path)
        .map_err(|e| {
            AppError::WarpInstaller(format!(
                "Failed to create secure script file in temp: {}",
                e
            ))
        })?;

    file.write_all(script_content.as_bytes()).map_err(|e| {
        AppError::WarpInstaller(format!("Failed to write interactive script: {}", e))
    })?;

    // Detect terminal emulator and run the script
    if let Some((term, mut args)) = find_terminal() {
        println!(
            "[WARP Installer] Found terminal emulator '{}'. Spawning installer window...",
            term
        );
        args.push(script_path_str.to_string());
        if let Err(e) = StdCommand::new(&term).args(args).spawn() {
            let _ = std::fs::remove_file(&script_path); // Cleanup on spawn failure
            return Err(AppError::WarpInstaller(format!(
                "Failed to launch terminal '{}': {}",
                term, e
            )));
        }

        Ok(
            "Terminal opened successfully. Please complete the steps in the new window."
                .to_string(),
        )
    } else {
        let _ = std::fs::remove_file(&script_path); // Cleanup on no terminal
        let err_msg = "No suitable terminal emulator (gnome-terminal, konsole, xterm, etc.) was found on your system!".to_string();
        eprintln!("[WARP Installer] Error: {}", err_msg);
        Err(AppError::WarpInstaller(err_msg))
    }
}

/// Enables or disables Cloudflare WARP.
/// If connect is true -> runs `warp-cli connect`
/// If connect is false -> runs `warp-cli disconnect`
pub async fn warp_toggle(connect: bool) -> Result<String, AppError> {
    let action = if connect { "connect" } else { "disconnect" };
    let output = Command::new("warp-cli")
        .arg(action)
        .output()
        .await
        .map_err(|e| AppError::WarpControl(format!("Failed to execute warp-cli command: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::WarpControl(format!(
            "WARP control error: {}",
            stderr
        )))
    }
}

/// Retrieves the connection status of Cloudflare WARP.
/// Runs `warp-cli status` and parses the output.
pub async fn get_warp_status() -> Result<String, AppError> {
    let output_result = Command::new("warp-cli").arg("status").output().await;

    let output = match output_result {
        Ok(o) => o,
        Err(e) => {
            // If warp-cli is not found on the system
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok("Not Installed".to_string());
            }
            return Err(AppError::WarpStatus(format!(
                "Error invoking warp-cli: {}",
                e
            )));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::WarpStatus(format!(
            "Could not get WARP status: {}",
            stderr
        )));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut status = "Disconnected".to_string(); // Default to disconnected

    for line in stdout_str.lines() {
        let trimmed = line.trim();
        // Parse status from the line starting with "Status update:"
        if let Some(suffix) = trimmed.strip_prefix("Status update:") {
            status = suffix.trim().to_string();
            break;
        }
    }

    Ok(status)
}

/// Retrieves the current operating mode of WARP.
/// Runs `warp-cli settings list` and parses the "Mode:" line.
pub async fn get_warp_mode() -> Result<String, AppError> {
    let output = Command::new("warp-cli")
        .args(["settings", "list"])
        .output()
        .await
        .map_err(|e| AppError::WarpStatus(format!("Failed to call settings list: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::WarpStatus(format!(
            "Error fetching WARP settings: {}",
            err_msg
        )));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if let Some(idx) = trimmed.find("Mode:") {
            let mode_str = trimmed[idx + 5..].trim();
            let lower = mode_str.to_lowercase();
            // Check complex modes containing both Warp and DoH first
            if lower.contains("warp")
                && (lower.contains("doh")
                    || lower.contains("dnsoverhttps")
                    || lower.contains("dns-over-https"))
            {
                return Ok("warp+doh".to_string());
            } else if lower.contains("doh")
                || lower.contains("dnsoverhttps")
                || lower.contains("dns-over-https")
            {
                return Ok("doh".to_string());
            } else if lower.contains("warp") {
                return Ok("warp".to_string());
            } else {
                return Ok(lower);
            }
        }
    }
    Ok("unknown".to_string())
}

/// Configures a new operating mode for WARP.
/// Runs `warp-cli mode <mode>`
pub async fn set_warp_mode(mode: &str) -> Result<String, AppError> {
    let output = Command::new("warp-cli")
        .args(["mode", mode])
        .output()
        .await
        .map_err(|e| AppError::WarpControl(format!("Failed to execute warp-cli command: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::WarpControl(format!(
            "WARP mode setting error: {}",
            stderr
        )))
    }
}
