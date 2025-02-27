use std::io::Read;
use std::env;
use std::fs;
use regex::{Regex, Captures};
use crate::colors;

fn format_data(key: &str, value: &str) -> String {
    format!("{color1}▪{bold} {key:7}{reset} {color2}{value}",
            key = key,
            value = value,
            color1 = colors::yellow,
            color2 = colors::cyan,
            bold = colors::bold,
            reset = colors::reset,
            )
}

// Search with Regex in a string and return all of the matches
fn match_regex(search_str: &str, regex: String) -> Option<Captures> {
    let re = Regex::new(&regex).unwrap();

    re.captures(search_str)
}

pub fn get_user_host_name() -> Result<(String, String), String> {
    // Username
    let username_env = env::var_os("USER");
    let username: String;

    if let Some(..) = username_env {
        username = username_env.unwrap().into_string().unwrap();
    } else {
        username = String::new();
    }

    // Hostname
    let mut buf = [0u8; 128];
    let hostname_cstr = nix::unistd::gethostname(&mut buf)
        .map_err(|_| "Failed getting hostname".to_owned())?;
    let hostname = hostname_cstr.to_str().map_err(|_| "Failed decoding hostname")?;

    // Combine username and hostname into a formatted string
    let main_color: &str;
    let second_color: &str;
    main_color = colors::yellow;
    second_color = colors::cyan;

    let user_host_name = format!("{color}{bold}{user}{reset}
                                 {bold}{color2}@{reset}{bold}{color}{host}{reset}",
                                 user = username,
                                 host = hostname,
                                 color = main_color,
                                 color2 = second_color,
                                 bold = colors::bold,
                                 reset = colors::reset,
                                 ).replace(" ", "").replace("\n", "");

    // Separator
    // format: username length + @ (1) + hostname length

    let user_host_name_len = username.len() + 1 + hostname.len();
    let mut separator = String::new();

    separator += colors::cyan;

    for _i in 0..(user_host_name_len) {
        separator += "-";
    }
    separator += colors::reset;

    Ok((user_host_name, separator))
}

pub fn get_distro_name() -> Result<String, String> {
    // First get the lsb-release file
    let lsb_release = fs::File::open("/etc/lsb-release");
    let mut buffer = String::new();

    // Check if lsb_release exists
    if let Ok(..) = lsb_release {
        // Read lsb_release into buffer
        let mut lsb_release = lsb_release.unwrap();
        let result = lsb_release.read_to_string(&mut buffer);

        if result.is_err() { return Err("error".to_string()); }

        // Match regex in buffer
        let re_lsb = match_regex(&buffer,
                                 r#"(?x)
                                 DISTRIB_DESCRIPTION=
                                 "?   # Quotes if description is multiple words
                                 (?P<distro_name>[^\n"]+)
                                 "?   # Ditto
                                 \n
                                 "#.to_string());

        // Check if regex matches
        if let Some(..) = re_lsb {
            let re_lsb = re_lsb.unwrap();

            let distro_name = re_lsb.name("distro_name")
                .unwrap()
                .as_str();
            return Ok(format_data("os", distro_name));
        }
    }

    // If no lsb-release then fetch os-release
    let os_release = fs::File::open("/etc/os-release");

    if os_release.is_err() {
        return Err("Error".to_string());
    }

    let mut os_release = os_release.unwrap();
    let result = os_release.read_to_string(&mut buffer);

    if result.is_err() { return Err("error".to_string()); }

    let re_os = match_regex(&buffer,
                            r#"(?x)
                            NAME=
                            "?   # Quotes if description is multiple words
                            (?P<distro_name>[^\n"]+)
                            "?   # Ditto
                            \n
                            "#.to_string()
    );

    if let Some(..) = re_os {
        let re_os = re_os.unwrap();

        let distro_name = re_os.name("distro_name")
            .unwrap()
            .as_str();
        return Ok(format_data("os", distro_name));
    }

    Err("error".to_string())
}

pub fn get_kernel(show_kern_name: bool) -> Result<String, String> {
    let uname = nix::sys::utsname::uname();
    Ok(format_data(
        "kernel",
        &if show_kern_name {
            format!("{}/{}", uname.sysname(), uname.machine())
        } else {
            format!("{}", uname.release())
        }))
}

pub fn get_shell() -> Result<String, String> {
    let shell_env = env::var_os("SHELL");

    if shell_env.is_none() {
        return Err("Error".to_string());
    }

    let shell = shell_env.unwrap().into_string().unwrap();

    let re_shell = match_regex(&shell,
                               r#"(?x)
                               (?P<shell_name>[^/]+)$
                               "#.to_string());

    if re_shell.is_none() {
        return Err("Error".to_string());
    }

    let re_shell = re_shell.unwrap();

    let shell = re_shell.name("shell_name").unwrap().as_str();
    Ok(format_data("shell", shell))
}

pub fn format_uptime(time: std::time::Duration) -> String {
    let uptime_seconds = time.as_secs();

    // Calculate the uptime in hours and minutes respectively
    let uptime_hours = uptime_seconds / (60 * 60);
    let uptime_minutes = (uptime_seconds % (60 * 60)) / 60;

    format_data(
        "uptime",
        &format!("{hours}h {minutes}m",
                 hours = uptime_hours,
                 minutes = uptime_minutes))
}

pub fn format_memory(mem: systemstat::Memory) -> String {
    format_data(
        "memory",
        &format!("{used} / {total}",
                 used = systemstat::saturating_sub_bytes(mem.total, mem.free),
                 total = mem.total))
}

pub fn format_battery(battery: systemstat::BatteryLife) -> String {
    format_data(
        "battery",
        &format!("{percent}%, {hours}h {minutes}m remaining",
                 percent = (battery.remaining_capacity * 100.0).trunc(),
                 hours = battery.remaining_time.as_secs() / 3600,
                 minutes = battery.remaining_time.as_secs() % 60))
}
