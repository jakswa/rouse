use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread::JoinHandle;

#[derive(Deserialize)]
struct CmdList {
    cmds: Vec<CmdCfg>,
}

#[derive(Deserialize, Default)]
struct CmdCfg {
    cmd: String,
    label: Option<String>,
}

fn main() {
    let mut children = build_config()
        .cmds
        .into_iter()
        .map(|cmd_cfg| {
            let title = cmd_cfg
                .label
                .unwrap_or(cmd_cfg.cmd.split(" ").next().unwrap().to_string());
            let mut cmd = Command::new("sh");
            cmd.arg("-c");
            cmd.arg(cmd_cfg.cmd.clone());

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

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

fn build_config() -> CmdList {
    let run_args = std::env::args();
    if run_args.len() > 1 {
        let cmd_list = run_args
            .skip(1)
            .map(|run_arg| CmdCfg {
                cmd: run_arg.clone(),
                label: None,
            })
            .collect::<Vec<CmdCfg>>();
        return CmdList { cmds: cmd_list };
    } else if let Ok(toml_str) = std::fs::read_to_string("./cmds.toml") {
        return toml::from_str::<CmdList>(&toml_str).unwrap();
    }
    panic!("was not able to figure out what you want to run")
}
