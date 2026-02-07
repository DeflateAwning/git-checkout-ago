use std::env;
use std::error::Error;
use std::process::Command;

/// Convert shorthand like `2d`, `3h`, `1w` into git-compatible strings.
/// If the input doesn't match shorthand, return it unchanged.
fn normalize_ago(input: &str) -> String {
    let input = input.trim();

    if input.is_empty() {
        return input.to_string();
    }

    let (number, unit) = input.split_at(
        input
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(input.len()),
    );

    if number.is_empty() || unit.is_empty() {
        return input.to_string();
    }

    let expanded_unit = match unit {
        "s" => "seconds",
        "m" => "minutes",
        "h" => "hours",
        "d" => "days",
        "w" => "weeks",
        _ => return input.to_string(),
    };

    format!("{number} {expanded_unit}")
}

/// Build the `git rev-list` command arguments for a given "ago" string.
fn rev_list_args(ago: &str) -> Vec<String> {
    let ago = normalize_ago(ago);

    vec![
        "rev-list".into(),
        "-n".into(),
        "1".into(),
        format!("--before={} ago", ago),
        "HEAD".into(),
    ]
}

/// Build the `git checkout` command arguments.
fn checkout_args(commit: &str) -> Vec<String> {
    vec!["checkout".into(), commit.into()]
}

/// Core logic, split out for testability.
fn run(ago: &str) -> Result<(), Box<dyn Error>> {
    let rev_args = rev_list_args(ago);

    let output = Command::new("git").args(&rev_args).output()?;

    if !output.status.success() {
        return Err("git rev-list failed".into());
    }

    let commit = String::from_utf8(output.stdout)?.trim().to_string();

    if commit.is_empty() {
        return Err("no commit found before the given time".into());
    }

    let checkout = Command::new("git").args(checkout_args(&commit)).status()?;

    if !checkout.success() {
        return Err("git checkout failed".into());
    }

    Ok(())
}

fn main() {
    let mut args = env::args().skip(1);

    let Some(ago) = args.next() else {
        eprintln!("usage: checkout-ago <time>");
        eprintln!(
            r#"examples:
  checkout-ago "2 days"
  checkout-ago 2d
  checkout-ago 3h"#
        );
        std::process::exit(1);
    };

    if let Err(e) = run(&ago) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_shorthand_days() {
        assert_eq!(normalize_ago("2d"), "2 days");
    }

    #[test]
    fn test_normalize_shorthand_hours() {
        assert_eq!(normalize_ago("3h"), "3 hours");
    }

    #[test]
    fn test_normalize_shorthand_weeks() {
        assert_eq!(normalize_ago("1w"), "1 weeks");
    }

    #[test]
    fn test_normalize_shorthand_minutes() {
        assert_eq!(normalize_ago("15m"), "15 minutes");
    }

    #[test]
    fn test_normalize_shorthand_seconds() {
        assert_eq!(normalize_ago("30s"), "30 seconds");
    }

    #[test]
    fn test_normalize_passthrough() {
        assert_eq!(normalize_ago("2 days"), "2 days");
        assert_eq!(normalize_ago("1 week"), "1 week");
    }

    #[test]
    fn test_normalize_invalid_unit() {
        assert_eq!(normalize_ago("10x"), "10x");
    }

    #[test]
    fn test_rev_list_args() {
        let args = rev_list_args("2 days");

        assert_eq!(
            args,
            vec!["rev-list", "-n", "1", "--before=2 days ago", "HEAD"]
        );
    }

    #[test]
    fn test_rev_list_args_with_shorthand() {
        let args = rev_list_args("2d");

        assert_eq!(
            args,
            vec!["rev-list", "-n", "1", "--before=2 days ago", "HEAD"]
        );
    }

    #[test]
    fn test_checkout_args() {
        let args = checkout_args("abc123");

        assert_eq!(args, vec!["checkout", "abc123"]);
    }

    #[test]
    fn test_empty_ago_string() {
        let args = rev_list_args("");
        assert_eq!(args[3], "--before= ago");
    }
}
