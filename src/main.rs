use std::env;
use std::error::Error;
use std::process::Command;

/// Build the `git rev-list` command arguments for a given "ago" string.
fn rev_list_args(ago: &str) -> Vec<String> {
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

    let checkout = Command::new("git").args(&checkout_args(&commit)).status()?;

    if !checkout.success() {
        return Err("git checkout failed".into());
    }

    Ok(())
}

fn main() {
    let mut args = env::args().skip(1);

    let ago = match args.next() {
        Some(a) => a,
        None => {
            eprintln!("usage: checkout-ago <time>");
            eprintln!(r#"example: checkout-ago "2 days""#);
            std::process::exit(1);
        }
    };

    if let Err(e) = run(&ago) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rev_list_args() {
        let args = rev_list_args("2 days");

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
