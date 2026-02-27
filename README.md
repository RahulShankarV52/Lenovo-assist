# Lenovo Assist
A lightning-fast, memory-safe, root-separated hardware daemon for Lenovo laptops (Ideapad/Legion) running Linux. Built entirely in Rust.
Lenovo Assist replaces cinnamon desktop applets with a simpler system tray utility. It dynamically monitors and controls ACPI hardware states like Conservation Mode, Fn Lock, and the physical Camera Privacy Shutter.
## Features
* **Zero-Overhead Polling:** Uses a detached OS-level heartbeat thread and native Linux `sysfs` reads to achieve 0.0% idle CPU usage.
* **Dynamic Hardware Discovery:** Automatically hunts down and maps ACPI paths, ensuring compatibility across different Lenovo models and future Linux kernel updates.
* **Bulletproof Status Indicators:** The dynamic system tray icon uses a custom 3-LED pixel array to visually track the hardware state in real-time.
* **Native Desktop Integration:** Fires clean, native Cinnamon/GTK desktop notifications for hardware interrupts.
## Architecture & Security
This workspace is designed with strict privilege separation:
1. **The Core (`lenovo-assist`):** A headless CLI binary that requires `root` to modify `/sys/` hardware files.
2. **The Tray (`lenovo-assist-tray`):** A lightweight GUI daemon that runs entirely in user-space.
3. **Polkit Integration:** Uses custom `polkit` rules to allow the tray to securely execute hardware toggles via `pkexec` without ever prompting for a `sudo` password.
## Installation & Build Instructions
### Prerequisites
You will need the Rust toolchain and standard Linux UI headers:
```bash
sudo apt install build-essential libgtk-3-dev libayatana-appindicator3-dev
```
1. Compile the Workspace
Clone the repository and build the release versions. Cargo will compile both the core and tray simultaneously:

```Bash
cargo build --release
sudo cp target/release/lenovo-assist target/release/lenovo-assist-tray /usr/local/bin/
```
2. Set up Polkit (Passwordless Toggling)
To allow the tray to toggle hardware without a password prompt:
```Bash
sudo nano /etc/polkit-1/rules.d/99-lenovo-assist.rules
```
Paste the following:
```JavaScript
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.policykit.exec" &&
        action.lookup("program") == "/usr/local/bin/lenovo-assist" &&
        subject.isInGroup("sudo")) {
        return polkit.Result.YES;
    }
});
```
3. Hardware Camera Notifications (ACPI)
To get native desktop notifications when you slide the physical camera privacy shutter, map the core binary to your ACPI daemon:
```Bash
sudo nano /etc/acpi/events/lenovo-camera
```
Add these lines:
```TOML
event=video/webcam.*
action=/usr/local/bin/lenovo-assist camera-notify
```
Restart the ACPI daemon: `sudo systemctl restart acpid`
4. Autostart on Login
Create a .desktop entry to launch the tray seamlessly on boot:
```Bash
nano ~/.config/autostart/lenovo-assist.desktop
```
Add this to it
```Toml
[Desktop Entry]
Type=Application
Name=Lenovo Assist
Exec=bash -c "sleep 5 && /usr/local/bin/lenovo-assist-tray"
Icon=preferences-system-hardware
Terminal=false
Categories=System;Utility;
```
## Command Line Usage
The core binary can be mapped directly to keyboard shortcuts (like Ulauncher) for instant, headless toggling:
```
lenovo-assist battery --quiet
lenovo-assist fnlock --quiet
```
---
