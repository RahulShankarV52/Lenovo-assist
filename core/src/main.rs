use std::env::args;
use std::fs::{self, write};
use std::io::{Error, ErrorKind};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

fn find_ideapad_path() -> Option<PathBuf> {
    let base_dir = "/sys/bus/platform/drivers/ideapad_acpi/";
    if let Ok(entries) = fs::read_dir(base_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                let test_file = path.join("conservation_mode");
                if test_file.is_file() {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn get_active_user() -> Option<(u32, String)> {
    // Changed String to u32 here
    let entries = fs::read_dir("/run/user").ok()?;
    for entry in entries.filter_map(Result::ok) {
        let uid_str = entry.file_name().into_string().unwrap_or_default();
        if let Ok(uid) = uid_str.parse::<u32>() {
            if uid >= 1000 {
                let output = Command::new("id").arg("-un").arg(&uid_str).output().ok()?;
                let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Some((uid, username));
            }
        }
    }
    None
}

fn send_desktop_notification(icon: &str, title: &str, message: &str) {
    let (uid, _username) = match get_active_user() {
        Some(user_data) => user_data,
        None => return,
    };
    Command::new("notify-send")
        .uid(uid)
        .env("DISPLAY", ":0")
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path=/run/user/{}/bus", uid),
        )
        .arg("-u")
        .arg("normal")
        .arg("-t")
        .arg("2000")
        .arg("-h")
        .arg("string:x-canonical-private-synchronous:lenovo")
        .arg("-i")
        .arg(icon)
        .arg(title)
        .arg(message)
        .output()
        .ok();
}

fn handle_camera_notification() {
    thread::sleep(Duration::from_millis(500));

    let output = match Command::new("v4l2-ctl")
        .arg("-d")
        .arg("/dev/video0")
        .arg("--get-ctrl=privacy")
        .output()
    {
        Ok(out) => out,
        Err(_) => return,
    };

    let privacy_str = String::from_utf8_lossy(&output.stdout);
    let trimmed = privacy_str.trim();

    let (icon, title, message) = if trimmed.ends_with('1') {
        (
            "camera-disabled-symbolic",
            "Camera Privacy",
            "Camera is BLOCKED",
        )
    } else if trimmed.ends_with('0') {
        ("camera-web-symbolic", "Camera Privacy", "Camera is LIVE")
    } else {
        (
            "camera-web-symbolic",
            "Camera Switch",
            "Privacy toggle detected (Status Unknown)",
        )
    };

    send_desktop_notification(icon, title, message);
}

fn read_sysfs_toggle(path: &Path) -> Result<bool, std::io::Error> {
    let file = fs::read_to_string(path)?;
    match file.trim() {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            "Unexpected data in sysfs file",
        )),
    }
}

fn write_sysfs_toggle(path: &Path, enable: bool) -> Result<(), std::io::Error> {
    let value_to_write = if enable { "1" } else { "0" };
    write(path, value_to_write)?;
    Ok(())
}

fn toggle_feature(path: &Path, feature: &str, icon_name: &str, quiet: bool) {
    let current_state = match read_sysfs_toggle(path) {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Failed to read hardware state: {}", e);
            return;
        }
    };

    let new_state = !current_state;

    if let Err(e) = write_sysfs_toggle(path, new_state) {
        eprintln!("Failed to write to sysfs: {}", e);
        return;
    }

    let state_str = if new_state { "ON" } else { "OFF" };

    if !quiet {
        let message = format!("{} is now {}", feature, state_str);
        send_desktop_notification(icon_name, feature, &message);
        println!("Success! {} is {}", feature, state_str);
    }
}

fn main() {
    let hw_path = match find_ideapad_path() {
        Some(path) => path,
        None => {
            eprintln!("Error: Lenovo ACPI driver not found. Is this an Ideapad/Legion?");
            return;
        }
    };

    let conservation_path = hw_path.join("conservation_mode");
    let fnlock_path = hw_path.join("fn_lock");

    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        println!("Usage: lenovo-assist [battery|fnlock|camera-notify] [--quiet]");
        return;
    }

    let quiet = args.iter().any(|a| a == "--quiet");

    match args[1].as_str() {
        "battery" => toggle_feature(
            &conservation_path,
            "Conservation Mode",
            "battery-good-symbolic",
            quiet,
        ),
        "fnlock" => toggle_feature(&fnlock_path, "Fn Lock", "keyboard-symbolic", quiet),
        "camera-notify" => handle_camera_notification(),
        _ => println!("Feature not found. Use 'battery', 'fnlock', or 'camera-notify'."),
    }
}
