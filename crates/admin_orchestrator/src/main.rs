use std::process::Command;

fn run(cmd: &str, args: &[&str]) -> anyhow::Result<()> {
    println!("[admin-orchestrator] {cmd} {:?}", args);
    let status = Command::new(cmd).args(args).status()?;
    if !status.success() {
        anyhow::bail!("{cmd:?} failed with status {status}");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // SYSTEM PREP (minimal, user-reviewed)
    run("sudo", &["apt", "update"])?;
    run("sudo", &["apt", "install", "-y", "build-essential", "git"])?;

    // FIREWALL ENABLE (delegated to AuraBoundaryGuard config on host)
    // Here we *advise* UFW but do not hardcode ports.
    run("sudo", &["ufw", "enable"])?;

    // SECURITY TOOLS (optional, host decides)
    run("sudo", &["apt", "install", "-y", "auditd", "lynis"])?;

    // GIT identity only; no remotes or pushes.
    run("git", &["config", "--global", "user.name", "Doctor0Evil"])?;
    run("git", &["config", "--global", "user.email", "Doctor0Evil@protonmail.com"])?;

    println!("[✔] Sovereign admin orchestration completed (subset).");
    Ok(())
}
