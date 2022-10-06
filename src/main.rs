use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread::JoinHandle;

#[derive(Deserialize)]
struct CmdsToml {
    cmds: std::collections::HashMap<String, CmdCfg>,
}

#[derive(Deserialize)]
struct CmdCfg {
    args: Option<Vec<String>>,
    cmd: Option<String>,
}

fn main() {
    let toml_str = std::fs::read_to_string("./cmds.toml").unwrap();
    let config: CmdsToml = toml::from_str(&toml_str).unwrap();

    let mut children = config
        .cmds
        .into_iter()
        .map(|(title, cmd_cfg)| {
            let mut cmd = Command::new(cmd_cfg.cmd.unwrap_or(title.clone()));

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            if let Some(args) = cmd_cfg.args {
                args.iter().for_each(|arg| {
                    cmd.arg(arg);
                });
            }

            let mut child = cmd.spawn().expect("failed to execute");

            let err = BufReader::new(child.stderr.take().unwrap());
            let errtitle = title.clone();
            let stderr = std::thread::spawn(move || {
                err.lines().for_each(|line| {
                    eprintln!("[{}] {}", errtitle, line.unwrap());
                });
            });

            let out = BufReader::new(child.stdout.take().unwrap());
            let outtitle = title.clone();
            let stdout = std::thread::spawn(move || {
                out.lines().for_each(|line| {
                    println!("[{}] {}", outtitle, line.unwrap());
                });
            });

            (child, stdout, stderr)
        })
        .collect::<Vec<(Child, JoinHandle<()>, JoinHandle<()>)>>();

    for (mut child, stdout, stderr) in children.drain(..) {
        child.wait().expect("command errored");
        stdout.join().expect("stdout errored");
        stderr.join().expect("stderr errored");
    }
}
