use tauri_plugin_shell::ShellExt;

#[tauri::command]
pub fn start_video_in_mpc(filepath: &str) -> () {
    /* let shell = app_handle.shell();
    let output = tauri::async_runtime::block_on(async move {
        shell
            .command("echo")
            .args(["Hello from Rust!"])
            .output()
            .await
            .unwrap()
    });
    if output.status.success() {
        println!("Result: {:?}", String::from_utf8(output.stdout));
    } else {
        println!("Exit with code: {}", output.status.code().unwrap());
    }*/
    //let _ = Command::new("C:/Program Files (x86)/K-Lite Codec Pack/MPC-HC64/mpc-hc64.exe").arg(" ").to_owned() + filepath).spawn();
}