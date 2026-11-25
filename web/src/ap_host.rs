//! Archipelago's `host.yaml`: read and set `ootrr_options.ootrr_exe`, the path of the OOTRR app
//! the world and its client run. The file is edited as text, one line at a time, so its comments
//! and every other setting stay as Archipelago wrote them.

use std::path::Path as P;

pub const FILE: &str = "host.yaml";
pub const SECTION: &str = "ootrr_options";
pub const KEY: &str = "ootrr_exe";

fn top(line: &str) -> bool {
    line.chars().next().is_some_and(|c| !c.is_whitespace() && c != '#')
}

/// The line of the section's header, and the line after its last indented line.
fn block(lines: &[&str]) -> Option<(usize, usize)> {
    let head: usize = lines.iter().position(|l| l.strip_prefix(SECTION).is_some_and(|r| r.starts_with(':')))?;
    let end: usize = lines[head + 1..].iter().position(|l| top(l)).map(|k| head + 1 + k).unwrap_or(lines.len());
    Some((head, end))
}

fn key_line(lines: &[&str], head: usize, end: usize) -> Option<usize> {
    (head + 1..end).find(|&i| lines[i].trim_start().strip_prefix(KEY).is_some_and(|r| r.trim_start().starts_with(':')))
}

/// A YAML scalar as text: plain, 'single quoted' or "double quoted".
fn scalar(raw: &str) -> String {
    let raw: &str = raw.trim();
    if let Some(r) = raw.strip_prefix('\'') {
        let mut out: String = String::new();
        let mut it = r.chars().peekable();
        while let Some(c) = it.next() {
            if c == '\'' {
                if it.peek() == Some(&'\'') {
                    it.next();
                    out.push('\'');
                    continue;
                }
                break;
            }
            out.push(c);
        }
        return out;
    }
    if let Some(r) = raw.strip_prefix('"') {
        let mut out: String = String::new();
        let mut it = r.chars();
        while let Some(c) = it.next() {
            match c {
                '"' => break,
                '\\' => {
                    if let Some(n) = it.next() {
                        out.push(n);
                    }
                }
                _ => out.push(c),
            }
        }
        return out;
    }
    raw.split(" #").next().unwrap_or("").trim().to_string()
}

fn quoted(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// The value of `ootrr_options.ootrr_exe`, when the file sets it.
pub fn exe_in(yaml: &str) -> Option<String> {
    let lines: Vec<&str> = yaml.lines().collect();
    let (head, end) = block(&lines)?;
    let i: usize = key_line(&lines, head, end)?;
    let v: String = scalar(lines[i].split_once(':')?.1);
    if v.is_empty() { None } else { Some(v) }
}

/// The file with `ootrr_options.ootrr_exe` set to `exe`; everything else is kept.
pub fn with_exe(yaml: &str, exe: &str) -> String {
    let eol: &str = if yaml.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = yaml.lines().map(str::to_string).collect();
    let view: Vec<&str> = lines.iter().map(String::as_str).collect();
    match block(&view) {
        Some((head, end)) => match key_line(&view, head, end) {
            Some(i) => {
                let indent: String = lines[i].chars().take_while(|c| c.is_whitespace()).collect();
                lines[i] = format!("{indent}{KEY}: {}", quoted(exe));
            }
            None => {
                lines[head] = format!("{SECTION}:");
                lines.insert(head + 1, format!("  {KEY}: {}", quoted(exe)));
            }
        },
        None => {
            lines.push(format!("{SECTION}:"));
            lines.push(format!("  {KEY}: {}", quoted(exe)));
        }
    }
    let mut out: String = lines.join(eol);
    out.push_str(eol);
    out
}

/// True when both paths name the same file (the same text counts when neither can be resolved).
pub fn same_file(a: &P, b: &P) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => {
            let norm = |p: &P| p.to_string_lossy().replace('\\', "/").to_lowercase();
            norm(a) == norm(b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOST: &str = "general_options:\n  # Where to place output files\n  output_path: \"output\"\nserver_options:\n  port: 38281\n";

    #[test]
    fn a_file_without_the_section_gets_it_at_the_end() {
        assert_eq!(exe_in(HOST), None);
        let t: String = with_exe(HOST, r"C:\Games\OOTRR\ootrr.exe");
        assert!(t.starts_with(HOST));
        assert!(t.ends_with("ootrr_options:\n  ootrr_exe: 'C:\\Games\\OOTRR\\ootrr.exe'\n"));
        assert_eq!(exe_in(&t).as_deref(), Some(r"C:\Games\OOTRR\ootrr.exe"));
        assert_eq!(with_exe(&t, r"C:\Games\OOTRR\ootrr.exe"), t);
    }

    #[test]
    fn an_existing_value_is_replaced_in_place_and_the_rest_is_kept() {
        let host: &str = "general_options:\r\n  output_path: \"output\"\r\nootrr_options:\r\n  # The OOTRR app\r\n  ootrr_exe: \"ootrr.exe\"\r\nserver_options:\r\n  port: 38281\r\n";
        assert_eq!(exe_in(host).as_deref(), Some("ootrr.exe"));
        let t: String = with_exe(host, "D:\\It's here\\ootrr.exe");
        assert_eq!(t, host.replace("\"ootrr.exe\"", "'D:\\It''s here\\ootrr.exe'"));
        assert_eq!(exe_in(&t).as_deref(), Some("D:\\It's here\\ootrr.exe"));
    }

    #[test]
    fn an_empty_section_and_an_empty_file_work_too() {
        let t: String = with_exe("ootrr_options: {}\nserver_options:\n  port: 1\n", "/opt/ootrr");
        assert_eq!(t, "ootrr_options:\n  ootrr_exe: '/opt/ootrr'\nserver_options:\n  port: 1\n");
        assert_eq!(with_exe("", "/opt/ootrr"), "ootrr_options:\n  ootrr_exe: '/opt/ootrr'\n");
    }

    #[test]
    fn scalars_read_in_all_three_styles() {
        for (line, want) in [
            ("ootrr_options:\n  ootrr_exe: C:\\a b\\ootrr.exe # the app\n", "C:\\a b\\ootrr.exe"),
            ("ootrr_options:\n  ootrr_exe: \"C:\\\\a\\\\ootrr.exe\"\n", "C:\\a\\ootrr.exe"),
            ("ootrr_options:\n  ootrr_exe: 'C:\\a\\it''s.exe'\n", "C:\\a\\it's.exe"),
        ] {
            assert_eq!(exe_in(line).as_deref(), Some(want), "{line}");
        }
        assert_eq!(exe_in("other_options:\n  ootrr_exe: x\n"), None);
        assert_eq!(exe_in("ootrr_options:\n  ootrr_exe:\nnext:\n  ootrr_exe: y\n"), None);
    }

    #[test]
    fn paths_compare_by_file_or_by_text() {
        let here: std::path::PathBuf = std::env::current_exe().unwrap();
        assert!(same_file(&here, &here));
        assert!(same_file(P::new("C:\\No\\Such\\ootrr.exe"), P::new("c:/no/such/OOTRR.exe")));
        assert!(!same_file(P::new("C:\\No\\Such\\ootrr.exe"), P::new("C:\\No\\Other\\ootrr.exe")));
    }
}
