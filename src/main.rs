use clap::Parser;
use std::error::Error;
use std::process::Command;

/// Checkout the most recent commit before a given time.
#[derive(Parser, Debug)]
#[command(
    name = "checkout-ago",
    about = "Check out the most recent git commit before a given time",
    long_about = None
)]
struct Cli {
    /// Time before now (e.g. "2 days", 2d, 3h, 1w)
    #[arg(value_name = "TIME")]
    ago: String,

    /// Only print where you are and where you would jump to
    #[arg(long, alias = "show")]
    print: bool,
}

fn current_head() -> Result<String, Box<dyn Error>> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;

    if !output.status.success() {
        return Err("git rev-parse failed".into());
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

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
fn run(ago: &str, print_only: bool) -> Result<(), Box<dyn Error>> {
    let original_head = current_head()?;

    let rev_args = rev_list_args(ago);
    let output = Command::new("git").args(&rev_args).output()?;

    if !output.status.success() {
        return Err("git rev-list failed".into());
    }

    let target = String::from_utf8(output.stdout)?.trim().to_string();

    if target.is_empty() {
        return Err("no commit found before the given time".into());
    }

    {
        println!("Current HEAD: {original_head}");
        println!("Target commit: {target}");
        println!("To return: git checkout {original_head}");
    }

    if !print_only {
        println!();
        let checkout = Command::new("git").args(checkout_args(&target)).status()?;

        if !checkout.success() {
            return Err("git checkout failed".into());
        }
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli.ago, cli.print) {
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
