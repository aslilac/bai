<h1 align="center">
bai
</h1>

![`basically-anything init`](https://cdn.mckayla.cloud/-/1cvf81e1yz6ffb/bai.svg)

A large collection of file templates, and a command line tool to make use of them!

Many of the files in this repository are highly opinionated, if you disagree with some of the opinions, that's fine! You can just fork the repository, edit the collection of templates to your liking, and edit the configuration to use your repository instead!

```sh
bai -set github.username=aslilac
bai /rs         # Will create a bunch of files for a project written in Rust!
bai LICENSE-MIT # Just use a single template file. Note that this will get
                # filled in with your name and the current year for you!

# Most templates use variables which will be filled in automatically based on
# the current directory, your Git configuration, and other such "system state".
# Defaults try to be sensible, but all of these variables can also be set manually!
bai Cargo.toml -define name="coolest_new_crate"
```
