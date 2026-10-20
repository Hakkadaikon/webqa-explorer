//! Steps of the trials on how the reset command is run: its arguments, what
//! it is given, and what it writes.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use cucumber::{given, then, when};
use webqa::interview::reset;
use webqa_core::config::interview::Reset;
use webqa_core::redact::Redactor;

use super::{fill, found};
use crate::run_clock::TrialClock;
use crate::scene::Scene;
use crate::world::InterviewWorld;

/// The script of the fixtures that stands for the one the feature names.
const SCRIPT: &str = "./reset-db.sh";

/// The file the reset command of a scenario leaves its records beside.
fn kept(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("trials");
    assert!(fs::create_dir_all(&dir).is_ok(), "the directory is made");
    // Runs of the tests at the same time do not share the files.
    dir.join(format!("{}-{name}", process::id()))
}

/// The file `extra` of the command that records into `out`.
fn beside(out: &Path, extra: &str) -> PathBuf {
    PathBuf::from(format!("{}.{extra}", out.display()))
}

/// Makes the reset command of `scene` record into the files beginning
/// `name`, and removes what an earlier run left.
fn recording(scene: &mut Scene, name: &str) {
    let out = kept(name);
    for extra in ["args", "env", "parent"] {
        drop(fs::remove_file(beside(&out, extra)));
    }
    scene.env.push(("RESET_OUT".into(), OsString::from(&out)));
}

/// The reset command of `scene` with the arguments `args`.
fn with_arguments(scene: &mut Scene, args: Vec<String>) {
    scene.reset = scene.reset.take().map(|reset| Reset { args, ..reset });
}

#[given(expr = "設定の初期化のコマンドが引数の列 [{string}, {string}] である")]
fn reset_arguments(world: &mut InterviewWorld, program: String, argument: String) {
    assert_eq!(program, SCRIPT, "the script the fixture stands for");
    let scene = &mut world.scene;
    scene.reset_with("record.sh", 5);
    recording(scene, "arguments");
    with_arguments(scene, vec![argument]);
    fill(scene);
    drop(program);
}

#[when(expr = "初期化のコマンドを実行する")]
async fn reset_run(world: &mut InterviewWorld) {
    world.run().await;
}

#[then(expr = "{string} が、2つ目の引数 {string} を1つの文字列として受け取って実行されている")]
async fn received_whole(world: &mut InterviewWorld, program: String, argument: String) {
    world.run().await;
    assert_eq!(program, SCRIPT, "the script the fixture stands for");
    let text = fs::read_to_string(beside(&kept("arguments"), "args"));
    let text = text.unwrap_or_default();
    let received: Vec<&str> = text.split_terminator('\0').collect();
    assert_eq!(received, [argument.as_str()], "the arguments received");
}

#[then(expr = "シェルは起動されていない")]
async fn no_shell(world: &mut InterviewWorld) {
    world.run().await;
    // The command was started by this process itself, not by a shell.
    let parent = fs::read_to_string(beside(&kept("arguments"), "parent"));
    assert_eq!(
        parent.unwrap_or_default(),
        process::id().to_string(),
        "the parent of the command"
    );
}

#[given(expr = "判断モデルの API キーは環境変数 {string} から読む設定である")]
fn key_from_variable(world: &mut InterviewWorld, name: String) {
    let scene = &mut world.scene;
    scene.reset_with("record.sh", 5);
    recording(scene, "environment");
    scene.env.push((OsString::from(&name), "the-key".into()));
    scene.env.push(("COPY".into(), "Bearer the-key".into()));
    scene.api_key_env = Some(name);
    fill(scene);
}

/// The names of the variables a command listed with `export -p`.
fn exported(text: &str) -> Vec<String> {
    let name = |line: &str| {
        let before = line.split('=').next().unwrap_or_default();
        before.rsplit(' ').next().unwrap_or_default().to_owned()
    };
    text.lines().map(name).collect()
}

#[then(expr = "初期化のコマンドの環境変数に {string} は含まれない")]
async fn key_not_passed(world: &mut InterviewWorld, name: String) {
    world.run().await;
    assert_eq!(world.ran().config_error, None, "the key was read");
    let text = fs::read_to_string(beside(&kept("environment"), "env"));
    let names = exported(&text.unwrap_or_default());
    let count = |wanted: &str| names.iter().filter(|listed| *listed == wanted).count();
    // The variable the value was copied into is withheld too; RESET_OUT shows the command ran.
    let passed = (count(&name), count("COPY"), count("RESET_OUT"));
    assert_eq!(passed, (0, 0, 1), "the variables the command got");
}

#[given(expr = "初期化のコマンドが標準出力に {string} と書く")]
fn reset_writes(world: &mut InterviewWorld, text: String) {
    let scene = &mut world.scene;
    scene.reset_with("echo.sh", 5);
    with_arguments(scene, vec![text]);
}

#[when(expr = "初期化のコマンドの出力をログに残す")]
async fn reset_output_logged(world: &mut InterviewWorld) {
    let scene = &world.scene;
    let redactor = found(Redactor::new(&scene.redact).ok(), "the rules read");
    let reset = found(world.scene.reset.take(), "the command is written");
    let mut clock = TrialClock {
        stall_sleep: true,
        ..TrialClock::default()
    };
    let ran = reset::run(&reset, &[], &[], &redactor, &mut clock).await;
    let logged = ran.output.stdout.as_inner();
    assert!(fs::write(kept("logged"), logged).is_ok(), "the log is kept");
}

#[then(expr = "ログには {string} が無く、伏せる対象の名前が書かれている")]
fn output_hidden(_world: &mut InterviewWorld, address: String) {
    let logged = fs::read_to_string(kept("logged")).unwrap_or_default();
    assert!(!logged.contains(&address), "the address is not in the log");
    assert_eq!(
        logged, "reset for [[email]]\n",
        "the name of the rule instead"
    );
    drop(address);
}
