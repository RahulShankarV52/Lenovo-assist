use std::fs;
use std::process::Command;
use std::time::Duration;
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{CheckMenuItem, Menu, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};

// --- READ STATE HELPERS ---

fn find_ideapad_path() -> Option<String> {
    let base_dir = "/sys/bus/platform/drivers/ideapad_acpi/";
    if let Ok(entries) = fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let test_file = path.join("conservation_mode");
                if test_file.exists() {
                    return Some(path.to_string_lossy().into_owned());
                }
            }
        }
    }
    None
}

fn read_sysfs_state(path: &str) -> bool {
    if let Ok(file) = fs::read_to_string(path) {
        return file.trim() == "1";
    }
    false
}

fn is_camera_blocked() -> bool {
    if let Ok(output) = Command::new("v4l2-ctl")
        .arg("-d")
        .arg("/dev/video0")
        .arg("--get-ctrl=privacy")
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.trim().ends_with('1');
    }
    false
}

fn generate_icon(cam_blocked: bool, bat_on: bool, fn_on: bool) -> tray_icon::Icon {
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let mut r = 40;
            let mut g = 40;
            let mut b = 40;
            let a = 255;

            if y >= 12 && y <= 20 {
                // LEFT LED (Camera)
                if x >= 2 && x <= 10 {
                    if cam_blocked {
                        r = 255;
                        g = 80;
                        b = 80;
                    } else {
                        r = 80;
                        g = 255;
                        b = 80;
                    }
                }
                // MIDDLE LED (Battery)
                else if x >= 12 && x <= 20 {
                    if bat_on {
                        r = 50;
                        g = 200;
                        b = 100;
                    } else {
                        r = 100;
                        g = 100;
                        b = 100;
                    }
                }
                // RIGHT LED (Fn Lock)
                else if x >= 22 && x <= 30 {
                    if fn_on {
                        r = 50;
                        g = 200;
                        b = 255;
                    } else {
                        r = 100;
                        g = 100;
                        b = 100;
                    }
                }
            }
            rgba.extend_from_slice(&[r, g, b, a]);
        }
    }
    tray_icon::Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}

// --- MAIN UI ---

fn main() {
    let hw_path = match find_ideapad_path() {
        Some(path) => path,
        None => {
            println!("Fatal: Lenovo hardware paths not found.");
            return;
        }
    };

    let bat_path = format!("{}/conservation_mode", hw_path);
    let fn_path = format!("{}/fn_lock", hw_path);

    let event_loop = EventLoopBuilder::new().build();
    let proxy = event_loop.create_proxy();

    // The Infallible Heartbeat Thread
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(1000));
        let _ = proxy.send_event(());
    });

    let tray_menu = Menu::new();

    let mut current_cam = is_camera_blocked();
    let mut current_bat = read_sysfs_state(&bat_path);
    let mut current_fn = read_sysfs_state(&fn_path);

    let camera_status = CheckMenuItem::new("Camera Privacy", false, current_cam, None);
    let toggle_battery = CheckMenuItem::new("Conservation Mode", true, current_bat, None);
    let toggle_fnlock = CheckMenuItem::new("Fn Lock", true, current_fn, None);
    let quit_item = PredefinedMenuItem::quit(None);

    tray_menu
        .append_items(&[
            &camera_status,
            &PredefinedMenuItem::separator(),
            &toggle_battery,
            &toggle_fnlock,
            &PredefinedMenuItem::separator(),
            &quit_item,
        ])
        .unwrap();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Lenovo Assist")
        .with_icon(generate_icon(current_cam, current_bat, current_fn))
        .build()
        .unwrap();

    let menu_channel = tray_icon::menu::MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::UserEvent(()) = event {
            let new_cam = is_camera_blocked();
            let new_bat = read_sysfs_state(&bat_path);
            let new_fn = read_sysfs_state(&fn_path);

            if new_cam != current_cam || new_bat != current_bat || new_fn != current_fn {
                current_cam = new_cam;
                current_bat = new_bat;
                current_fn = new_fn;

                camera_status.set_checked(current_cam);
                toggle_battery.set_checked(current_bat);
                toggle_fnlock.set_checked(current_fn);

                let _ =
                    tray_icon.set_icon(Some(generate_icon(current_cam, current_bat, current_fn)));
            }
        }

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == toggle_battery.id() {
                Command::new("pkexec")
                    .arg("/usr/local/bin/lenovo-assist")
                    .arg("battery")
                    .arg("--quiet")
                    .spawn()
                    .expect("Failed to run");
            } else if event.id == toggle_fnlock.id() {
                Command::new("pkexec")
                    .arg("/usr/local/bin/lenovo-assist")
                    .arg("fnlock")
                    .arg("--quiet")
                    .spawn()
                    .expect("Failed to run");
            }
        }

        if let Ok(_event) = tray_channel.try_recv() {}
    });
}
