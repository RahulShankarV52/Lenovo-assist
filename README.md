# Lenovo Assist

A lightning-fast, memory-safe, root-separated hardware daemon for Lenovo laptops (Ideapad/Legion) running Linux. Built entirely in Rust.

Lenovo Assist replaces bloated desktop applets with a simpler system tray utility. It dynamically monitors and controls ACPI hardware states like Conservation Mode, Fn Lock, and the physical Camera Privacy Shutter.

## Features
* **Zero-Overhead Polling:** Uses a detached OS-level heartbeat thread and native Linux `sysfs` reads to achieve 0.0% idle CPU usage.
* **Dynamic Hardware Discovery:** Automatically hunts down and maps ACPI paths, ensuring compatibility across different Lenovo models and future Linux kernel updates.
* **Bulletproof Status Indicators:** The dynamic system tray icon uses a custom 3-LED pixel array to visually track the hardware state in real-time.
* **Native Desktop Integration:** Fires clean, native Cinnamon/GTK desktop notifications for hardware interrupts.

---

## Installation (For Users)

If you are using a Debian-based distribution (Ubuntu, Linux Mint, Pop!_OS), the easiest way to install Lenovo Assist is using the pre-compiled `.deb` package. 

1. Go to the [Releases page](https://github.com/YOUR_USERNAME/Lenovo-assist/releases) and download the latest `lenovo-assist_amd64.deb` file.
2. Double-click the file to install it via your system's package manager, or run:
```bash
   sudo apt install ./lenovo-assist_*.deb
```
(Note: The .deb package automatically configures the necessary Polkit permissions and adds the tray application to your startup applications).

Hardware Camera Notifications (Optional)
To get native desktop notifications when you slide the physical camera privacy shutter, you must map the core binary to your ACPI daemon:
```Bash
sudo nano /etc/acpi/events/lenovo-camera
```
Add these lines:
```Ini, TOML
event=video/webcam.*
action=/usr/bin/lenovo-assist camera-notify
```
Restart the ACPI daemon: `sudo systemctl restart acpid`

## Command Line Usage
The core binary can be mapped directly to custom keyboard shortcuts (or launchers like Ulauncher) for instant, headless toggling:
```Bash
lenovo-assist battery --quiet
lenovo-assist fnlock --quiet
```
## Build from Source (For Developers)
### Architecture & Security
This workspace is designed with strict privilege separation:
**The Core (lenovo-assist)**: A headless CLI binary that requires root to modify /sys/ hardware files.
**The Tray (lenovo-assist-tray)**: A lightweight GUI daemon that runs entirely in user-space.

### Prerequisites
You will need the Rust toolchain and standard Linux UI headers:
```Bash
sudo apt install build-essential libgtk-3-dev libayatana-appindicator3-dev libxdo-dev
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
```
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.policykit.exec" &&
        action.lookup("program") == "/usr/local/bin/lenovo-assist" &&
        subject.isInGroup("sudo")) {
        return polkit.Result.YES;
    }
});
```
3. Autostart on Login
Create a .desktop entry to launch the tray seamlessly on boot:
```Bash
nano ~/.config/autostart/lenovo-assist.desktop
```
Paste the following:
```Ini, TOML
[Desktop Entry]
Type=Application
Name=Lenovo Assist
Exec=bash -c "sleep 5 && /usr/local/bin/lenovo-assist-tray"
Icon=preferences-system-hardware
Terminal=false
Categories=System;Utility;
```
Disclaimer: This is an unofficial, solo open-source project. It is not affiliated with, endorsed by, or supported by Lenovo.
