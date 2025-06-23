use std::{
    collections::HashMap,
    process::{Child, Command},
    sync::{Arc, Mutex},
};
use tauri::State;

// Global process storage
pub struct ProcessManager {
    pub processes: Mutex<HashMap<u32, Child>>, // Stores processes by ProcessID
}

pub(super) fn start_process(
    filepath: &str,
    args: &str,
    state: State<Arc<ProcessManager>>,
) -> Option<u32> {
    let process = Command::new(filepath).arg(args).spawn().ok()?; // Start process, return None on failure

    let process_id = process.id(); // Get process ID
    state.processes.lock().unwrap().insert(process_id, process);
    Some(process_id) // Return PID to JavaScript
}

fn stop_process(process_id: u32, state: State<Arc<ProcessManager>>) -> bool {
    if let Some(mut process) = state.processes.lock().unwrap().remove(&process_id) {
        let _ = process.kill(); // Attempt to kill the process
        return true;
    }
    false // Process not found
}

#[tauri::command(rename_all = "snake_case")]
pub fn is_process_running(process_id: u32, state: State<Arc<ProcessManager>>) -> bool {
    let mut processes = state.processes.lock().unwrap();

    if let Some(process) = processes.get_mut(&process_id) {
        // Try to get exit status - if it returns None, the process is still running
        match process.try_wait() {
            Ok(None) => true,     // Process is still running
            Ok(Some(_)) => false, // Process has exited
            Err(_) => false,      // Error checking process status
        }
    } else {
        false // Process not found in our HashMap
    }
}
