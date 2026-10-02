//! Parsing one `SUMMARY.md` into its rows.

/// One ``- `NAME` — DESCRIPTION`` row.
#[derive(Debug, PartialEq, Eq)]
pub struct Row {
    /// File name, folder name with a trailing `/`, or a glob.
    pub name: String,
}

/// Rows of `text`, or the problems that make it unreadable: a missing `# `
/// heading, a malformed list item, an empty description.
pub fn parse(text: &str) -> Result<Vec<Row>, Vec<String>> {
    let mut problems = Vec::new();
    if !text
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("# "))
    {
        problems.push("must start with a `# ` heading".to_owned());
    }
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if !line.starts_with("- ") {
            continue;
        }
        match parse_row(line) {
            Ok(row) => rows.push(row),
            Err(reason) => problems.push(format!("line {}: {reason}", index + 1)),
        }
    }
    if problems.is_empty() {
        Ok(rows)
    } else {
        Err(problems)
    }
}

fn parse_row(line: &str) -> Result<Row, String> {
    let malformed = || "expected ``- `NAME` — DESCRIPTION``".to_owned();
    let rest = line.strip_prefix("- `").ok_or_else(malformed)?;
    let (name, after) = rest.split_once('`').ok_or_else(malformed)?;
    let description = after
        .trim_start()
        .strip_prefix('—')
        .or_else(|| after.trim_start().strip_prefix("- "))
        .ok_or_else(malformed)?;
    if name.is_empty() {
        return Err("empty name".to_owned());
    }
    if description.trim().is_empty() {
        return Err(format!("`{name}` has an empty description"));
    }
    Ok(Row {
        name: name.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_rows_with_either_dash_and_skips_prose() {
        let rows = parse("# a\n\nText.\n\n- `x.rs` — the x\n- `d/` - a folder\n").unwrap();
        let names: Vec<_> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["x.rs", "d/"]);
    }

    #[test]
    fn reports_every_problem() {
        let problems = parse("no heading\n- `x` —\n- x.rs — no backticks\n").unwrap_err();
        assert_eq!(problems.len(), 3, "{problems:?}");
    }
}
