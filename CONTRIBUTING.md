
# Contributing to algorand-rs

**Note:** Any interaction with the project is subject to our [Code of Conduct](https://github.com/manuelmauro/algorand-rs/blob/main/CODE_OF_CONDUCT.md).

- [Contributing to algorand-rs](#contributing-to-algorand-rs)
  - [Submitting Issues](#submitting-issues)
  - [Pull Requests](#pull-requests)
    - [Submission Checklist](#submission-checklist)
  - [Writing Documentation](#writing-documentation)
  - [Useful Resources](#useful-resources)


## Submitting Issues

One way you can help `algorand-rs` is to report bugs or request features on our GitHub issue tracker.

## Pull Requests

We are very happy to accept code contributions too! Please address issues in our tracker or if you have any idea how to improve the project open a pull request.

### Submission Checklist

Before submitting your pull request to the repository, please make sure you have done the following things first:

1. You have ensured the pull request is based on a recent version of your respective branch.
2. You have processed your source code with the code formatter.
   1. `cargo fmt --all -- --check`
3. All of the following commands completed without errors or warnings.
   1. `cargo test`
   2. `cargo clippy -- -D warnings`

## Writing Documentation

Documentation improvements are always welcome! A solid project needs to have solid documentation to go with it. You can search for documentation-related issues on any of our GitLab trackers by filtering by the documentation label or submit an issue with your own suggestions for improvements.

## Useful Resources

- [Git Style Guide](https://github.com/agis/git-style-guide)
- [How to write a Git commit message](https://chris.beams.io/posts/git-commit/)
