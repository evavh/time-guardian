use std::fs;
use std::process::Command;

pub(crate) fn list_users() -> Vec<String> {
    fs::read_dir("/home")
        .unwrap()
        .map(Result::unwrap)
        .map(|d| d.file_name())
        .map(|s| s.into_string().unwrap())
        .collect()
}

pub(crate) fn logout(user: &str) {
    println!("Logging out user {user}");
    // Command::new("loginctl")
    //     .arg("terminate-user")
    //     .arg(user)
    //     .output()
    //     .unwrap();
}

pub(crate) fn exists(user: &str) -> bool {
    fs::read_to_string("/etc/passwd")
        .unwrap()
        .contains(&format!("{user}:"))
}

pub(crate) fn is_active(user: &str) -> bool {
    let output = Command::new("loginctl")
        .arg("show-user")
        .arg(user)
        .arg("--property=State")
        .output()
        .unwrap();

    let err = std::str::from_utf8(&output.stderr).unwrap();
    if !err.is_empty() {
        assert!(
            err.contains("is not logged in or lingering"),
            "Unknown loginctl error, output: {output:?}"
        );
    }
    let state = std::str::from_utf8(&output.stdout).unwrap();

    state.contains("active")
}
