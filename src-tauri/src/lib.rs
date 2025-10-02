use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PidInfo {
    pid: u32,
    ports: String,
    user: String,
    cpu: String,
    mem: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortInfo {
    process_name: String,
    command: String,
    pids: Vec<PidInfo>,
}

#[tauri::command]
fn list_ports() -> Result<Vec<PortInfo>, String> {
    #[cfg(debug_assertions)]
    println!("[DEBUG] list_ports command called");

    // Use -sTCP:LISTEN to only show listening ports (servers), not outbound connections
    let output = Command::new("lsof")
        .args(["-i", "-P", "-n", "-sTCP:LISTEN"])
        .output()
        .map_err(|e| {
            #[cfg(debug_assertions)]
            println!("[DEBUG] Failed to execute lsof: {}", e);
            format!("Failed to execute lsof: {}", e)
        })?;

    if !output.status.success() {
        #[cfg(debug_assertions)]
        println!("[DEBUG] lsof command failed with status: {}", output.status);
        return Err("lsof command failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut process_map: HashMap<u32, (String, Vec<String>, Vec<String>, String, String, String, String)> = HashMap::new();

    #[cfg(debug_assertions)]
    println!("[DEBUG] Parsing lsof output, {} lines", stdout.lines().count());

    for line in stdout.lines().skip(1) {
        // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let process_name = parts[0].to_string();
        let pid = parts[1].parse::<u32>().unwrap_or(0);
        let protocol = parts[7].to_string();
        let address = parts[8];

        // Extract port from address (format: *:PORT or IP:PORT)
        if let Some(port_str) = address.split(':').last() {
            // Filter out non-numeric ports
            if port_str.chars().all(|c| c.is_numeric()) {
                let entry = process_map.entry(pid).or_insert((
                    process_name.clone(),
                    Vec::new(),
                    Vec::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                ));
                if !entry.1.contains(&port_str.to_string()) {
                    entry.1.push(port_str.to_string());
                }
                if !entry.2.contains(&protocol) {
                    entry.2.push(protocol);
                }
            }
        }
    }

    // Get additional process info for each process
    for (pid, entry) in process_map.iter_mut() {
        // Get command, user, cpu, and memory - use column-based parsing
        if let Ok(ps_output) = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "user=,%cpu=,%mem=,command="])
            .output()
        {
            let ps_line = String::from_utf8_lossy(&ps_output.stdout).trim().to_string();

            // Split into at most 4 parts using whitespace
            let mut parts = Vec::new();
            let mut current = String::new();
            let mut word_count = 0;

            for word in ps_line.split_whitespace() {
                if word_count < 3 {
                    parts.push(word.to_string());
                    word_count += 1;
                } else {
                    if !current.is_empty() {
                        current.push(' ');
                    }
                    current.push_str(word);
                }
            }
            if !current.is_empty() {
                parts.push(current);
            }

            if parts.len() >= 4 {
                entry.4 = parts[0].clone(); // user
                entry.5 = parts[1].clone(); // %cpu
                entry.6 = parts[2].clone(); // %mem
                entry.3 = parts[3].clone(); // full command (everything after first 3 fields)
            }
        }
    }

    // Group by process name and command
    let mut process_groups: HashMap<(String, String), Vec<(u32, Vec<String>, String, String, String)>> = HashMap::new();

    for (pid, (process_name, mut port_list, _protocol_list, command, user, cpu, mem)) in process_map {
        port_list.sort_by_key(|p| p.parse::<u32>().unwrap_or(0));

        let key = (process_name.clone(), command.clone());
        process_groups.entry(key).or_insert_with(Vec::new).push((pid, port_list, user, cpu, mem));
    }

    // Convert to PortInfo structs
    let mut ports: Vec<PortInfo> = process_groups
        .into_iter()
        .map(|((process_name, command), mut pid_list)| {
            // Sort PIDs
            pid_list.sort_by_key(|(pid, _, _, _, _)| *pid);

            let pids = pid_list
                .into_iter()
                .map(|(pid, port_list, user, cpu, mem)| PidInfo {
                    pid,
                    ports: port_list.join(", "),
                    user,
                    cpu,
                    mem,
                })
                .collect();

            PortInfo {
                process_name,
                command,
                pids,
            }
        })
        .collect();

    // Sort by process name (case-insensitive)
    ports.sort_by(|a, b| a.process_name.to_lowercase().cmp(&b.process_name.to_lowercase()));

    #[cfg(debug_assertions)]
    println!("[DEBUG] Returning {} unique process groups", ports.len());

    Ok(ports)
}

#[tauri::command]
fn kill_process(pid: u32) -> Result<String, String> {
    #[cfg(debug_assertions)]
    println!("[DEBUG] Attempting to kill process with PID: {}", pid);

    let output = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()
        .map_err(|e| {
            #[cfg(debug_assertions)]
            println!("[DEBUG] Failed to execute kill command: {}", e);
            format!("Failed to kill process: {}", e)
        })?;

    #[cfg(debug_assertions)]
    println!("[DEBUG] Kill command exit status: {}", output.status);

    #[cfg(debug_assertions)]
    println!("[DEBUG] Kill command stdout: {}", String::from_utf8_lossy(&output.stdout));

    #[cfg(debug_assertions)]
    println!("[DEBUG] Kill command stderr: {}", String::from_utf8_lossy(&output.stderr));

    if output.status.success() {
        #[cfg(debug_assertions)]
        println!("[DEBUG] Process {} killed successfully", pid);
        Ok(format!("Process {} killed successfully", pid))
    } else {
        let err_msg = format!(
            "Failed to kill process {}: {}",
            pid,
            String::from_utf8_lossy(&output.stderr)
        );
        #[cfg(debug_assertions)]
        println!("[DEBUG] {}", err_msg);
        Err(err_msg)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_ports, kill_process])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
