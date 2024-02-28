use std::process::{Command, Stdio};

#[derive(Default, Debug)]
pub struct Diff {
    old_diff: Vec<DiffLine>,
    current_diff: Vec<DiffLine>,
}

impl Diff {
    pub fn parse_diff(diff_string: &str) -> Self {
        let lines = diff_string.split("\n");

        let mut diff = Self::default();

        let mut start = false;
        let mut additions = 0;
        let mut removals = 0;
        let mut diff_one_line = 1;
        let mut diff_two_line = 1;

        for line in lines {
            if line.starts_with("@@") && line.ends_with("@@") {
                start = true;
                continue;
            }

            if !start {
                continue;
            }

            let (prefix, content) = remove_first_char(line);

            match prefix {
                '+' => {
                    diff.current_diff.push(DiffLine::new(
                        content,
                        DiffKind::Addition,
                        Some(diff_two_line),
                    ));
                    diff_two_line += 1;
                    if removals > 0 {
                        removals -= 1
                    } else {
                        additions += 1
                    }
                }

                '-' => {
                    diff.old_diff.push(DiffLine::new(
                        content,
                        DiffKind::Removal,
                        Some(diff_one_line),
                    ));
                    diff_one_line += 1;
                    removals += 1
                }
                _ => {
                    for _ in 0..removals {
                        diff.current_diff
                            .push(DiffLine::new("", DiffKind::Blank, None))
                    }

                    removals = 0;

                    for _ in 0..additions {
                        diff.old_diff.push(DiffLine::new("", DiffKind::Blank, None))
                    }

                    additions = 0;

                    diff.old_diff.push(DiffLine::new(
                        content,
                        DiffKind::Neutral,
                        Some(diff_one_line),
                    ));
                    diff_one_line += 1;
                    diff.current_diff.push(DiffLine::new(
                        content,
                        DiffKind::Neutral,
                        Some(diff_two_line),
                    ));
                    diff_two_line += 1
                }
            }
        }

        for _ in 0..removals {
            diff.current_diff
                .push(DiffLine::new("", DiffKind::Blank, None))
        }

        for _ in 0..additions {
            diff.old_diff.push(DiffLine::new("", DiffKind::Blank, None))
        }

        diff
    }
    pub fn old_diff(&self) -> &[DiffLine] {
        &self.old_diff
    }

    pub fn current_diff(&self) -> &[DiffLine] {
        &self.current_diff
    }

    pub fn largest_line_number_len(&self) -> u16 {
        let largest_line_number = std::cmp::max(
            largest_line_number(&self.old_diff),
            largest_line_number(&self.current_diff),
        );

        let length = std::cmp::min(largest_line_number.to_string().len(), u16::MAX.into());
        length.try_into().unwrap_or(4)
    }
}

fn largest_line_number(diff: &[DiffLine]) -> usize {
    diff.iter()
        .map(|x| x.line_number().unwrap_or(0))
        .max()
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    content: String,
    kind: DiffKind,
    line_number: Option<usize>,
}

impl DiffLine {
    fn new(content: &str, kind: DiffKind, line_number: Option<usize>) -> Self {
        let content = content.to_string();
        Self {
            content,
            kind,
            line_number,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn kind(&self) -> &DiffKind {
        &self.kind
    }

    pub fn line_number(&self) -> &Option<usize> {
        &self.line_number
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DiffKind {
    Addition,
    Removal,
    Neutral,
    Blank,
}

impl DiffKind {
    pub fn value(&self) -> &str {
        match self {
            DiffKind::Addition => "+",
            DiffKind::Removal => "-",
            DiffKind::Neutral => " ",
            DiffKind::Blank => " ",
        }
    }
}

/// Performs 'git diff -U1000 <filename>' or 'git -C [path] diff -U1000 <filename>' and returns the result as a string
pub fn get_raw_diff(path: &str, dir_flag: bool) -> String {
    let args = if !dir_flag {
        ["diff", "-U1000", path, "", ""]
    } else {
        #[cfg(target_os = "windows")]
        let delimiter = '\\';
        #[cfg(not(target_os = "windows"))]
        let delimiter = '/';
        let (path, filename) = path.rsplit_once(delimiter).expect("Path to be valid");
        ["-C", path, "diff", "-U1000", filename]
    };

    // Process git diff <filename> command and save the stdout response
    let output = Command::new("git")
        .args(args)
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute process");

    // Convert stdout response to a string and return
    String::from_utf8(output.stdout).expect("UTF8 data to convert to string")
}

fn remove_first_char(string: &str) -> (char, &str) {
    let mut chars = string.chars();
    (chars.next().unwrap_or(' '), chars.as_str())
}
