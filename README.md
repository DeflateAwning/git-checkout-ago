# git-checkout-ago
`git checkout-ago 'x months'` alias to immediately browse a repo during its golden era (or like, before your oopsie)

## Usage

### Install
```bash
cargo install git-checkout-ago
```

### Use
```bash
cd some-git-repo

git checkout-ago '3 months'
git checkout-ago '2 days'
git checkout-ago '1 hour'
```

## Alternatives

Well, now this tool exists. Apparently built-in git features exist for this too though.

### Alternative 1

Add the following to your `~/.gitconfig`:

```
[alias]
    checkout-ago = "!f() { git checkout `git rev-list -n 1 --before=\"$1 ago\" HEAD`; }; f"
```

### Alternative 2

https://stackoverflow.com/questions/4052854/how-do-i-check-out-what-was-in-my-git-repository-n-days-ago

```bash
git checkout @{yesterday}
git checkout @{2.days.ago}
git checkout @{'2 days ago'}
git checkout @{'5 minutes ago'}
git checkout @{'1 month 2 weeks 3 days 1 hour 1 second ago'}
git checkout any-branch-name@{'1 hour ago'}
```
