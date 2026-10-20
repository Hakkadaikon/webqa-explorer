use std::cell::Cell;
use std::iter::once;
use std::thread;

use super::*;

const LEDGER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ledger.txt");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/truth.yaml");
const LABELS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/labels.yaml");
const SITE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/site");

/// Runs the command line; the exit code, the output and the error output.
fn run_with(argv: &[&str], hold: &dyn Fn(JoinHandle<()>)) -> (u8, String, String) {
    let (mut out, mut err) = (vec![], vec![]);
    let argv = once(&"webqa-eval").chain(argv).map(OsString::from);
    let code = entry(argv, &mut out, &mut err, hold);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

fn run_args(argv: &[&str]) -> (u8, String, String) {
    run_with(argv, &wait)
}

#[test]
fn score_prints_the_table_of_the_ledgers() {
    let (code, out, err) = run_args(&["score", "--truth", TRUTH, "--labels", LABELS, LEDGER]);
    assert_eq!(
        (code, out.lines().find(|l| l.starts_with("| 全体")), err),
        (0, Some("| 全体 | 0.67 | 0.67 | 1.00 |"), String::new())
    );
}

#[test]
fn score_names_the_file_it_cannot_read() {
    let missing = "no-such-truth.yaml";
    let (code, out, err) = run_args(&["score", "--truth", missing, "--labels", LABELS, LEDGER]);
    assert_eq!(
        (code, out, err),
        (
            2,
            String::new(),
            format!("{missing}: No such file or directory (os error 2)\n")
        )
    );
}

#[test]
fn score_names_the_file_that_is_not_a_ledger() {
    let (code, _, err) = run_args(&["score", "--truth", TRUTH, "--labels", LABELS, TRUTH]);
    assert_eq!(
        (code, err),
        (2, format!("{TRUTH}: 台帳の 1 行目が決まった書式でない\n"))
    );
}

#[test]
fn score_names_the_labels_file_that_is_not_labels() {
    let (code, out, err) = run_args(&["score", "--truth", TRUTH, "--labels", TRUTH, LEDGER]);
    assert_eq!(
        (code, out, err.starts_with(&format!("{TRUTH}: "))),
        (2, String::new(), true)
    );
}

#[test]
fn serve_prints_the_url_and_hands_the_server_over() {
    let held = Cell::new(0);
    let (code, out, err) = run_with(&["serve", SITE], &|handle| {
        held.set(held.get() + 1);
        drop(handle);
    });
    assert_eq!(
        (
            code,
            out.starts_with("http://127.0.0.1:"),
            out.ends_with("/\n"),
            err,
            held.get()
        ),
        (0, true, true, String::new(), 1)
    );
}

#[test]
fn help_goes_to_the_output_and_a_wrong_command_line_to_the_error_output() {
    let (help_code, help, _) = run_args(&["--help"]);
    let (code, out, err) = run_args(&["score"]);
    assert_eq!(
        (
            help_code,
            help.contains("Usage: webqa-eval"),
            code,
            out,
            err.contains("--truth")
        ),
        (0, true, 2, String::new(), true)
    );
}

#[test]
fn waiting_ends_with_the_thread() {
    let done = thread::spawn(|| {});
    wait(done);
}
