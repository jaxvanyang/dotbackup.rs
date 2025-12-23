# dotbackup.rs

> Rust implementation & successor of [dotbackup](https://github.com/jaxvanyang/dotbackup).

[![AUR version](https://img.shields.io/aur/version/dotbackup)](https://aur.archlinux.org/packages/dotbackup)
[![crate version](https://img.shields.io/crates/v/dotbackup)](https://crates.io/crates/dotbackup)

dotbackup is an easy-to-use yet powerful backup utility for dotfiles. Simplicity
is its main design principle. With its help, you don't have to maintain extra
backup scripts along with dotfiles. What you only need is adding dotfile
locations to its configuration. Flexibility is its another design principle.
With hooks, one can execute any script during the backup process. So you are not
limited to only copy dotfiles, you can use external commands to modify dotfiles
or anything you want in a clear and controllable way.

## Installation

Choose the appropriate package manager to install.

- [crates.io](https://crates.io/crates/dotbackup)
- [AUR](https://aur.archlinux.org/packages/dotbackup)

## Quick Start

dotbackup divides dotfile backup management into two separate operations -
backup and setup. So it has two commands - `dotbackup` and `dotsetup`.
`dotbackup` makes backup and `dotsetup` applies backup.

The first step is to create a configuration of dotbackup. Its location is
different on different platforms:

| Platform | Value | Example |
|:-|:-|:-|
| Linux | `$XDG_CONFIG_HOME/dotbackup/dotbackup.yml` or `$HOME/.config/dotbackup/dotbackup.yml` | `/home/alice/.config/dotbackup/dotbackup.yml` |
| macOS | `$HOME/Library/Application Support/dotbackup/dotbackup.yml` | `/Users/Alice/Library/Application Support/dotbackup/dotbackup.yml` |
| Windows | `{FOLDERID_RoamingAppData}` | `C:\Users\Alice\AppData\Roaming\dotbackup\dotbackup.yml` |

Let's say if you want to make backup of Vim and dotbackup itself, then the
configuration woule be like this:

```yml
backup_dir: ~/backup
apps:
  vim:
    files: [~/.vimrc]
  dotbackup:
    files: [~/.config/dotbackup/]
```

`backup_dir` is where backup files will be stored. Dotfile locations should be
specified in a structure of `apps.<app_name>.files`, so that dotbackup knows
which file belongs to which application. Details of the configuration are in the
documentation, but the example above should be enough for now.

After saving the configuration, you can make backup by simply execute the
command `dotbackup`. And then `~/.vimrc` will be copied to `~/backup/.vimrc`,
`~/.config/dotbackup/` will be copied to `~/backup/.config/dotbackup/`. The
backup location of files is determined by their relative paths to the home
directory. If you want to only make backup of Vim, just execute `dotbackup vim`.

Setup is just the inversion of backup. `dotsetup` applies all the backup, and
`dotsetup <app_name>` only applies specific application's backup.

## Documentation

For more information, please read [dotbackup(1)](docs/dotbackup.1.scdoc),
[dotsetup(1)](dosc/dotsetup.1.scdoc) and [dotbackup(5)](docs/dotbackup.5.scdoc).

## Show Your Support

If you're using dotbackup, consider adding the badge to your dotfile
repository's `README.md`:

```md
[![dotbackup-managed](https://img.shields.io/badge/dotbackup-managed-blue)](https://github.com/jaxvanyang/dotbackup.rs)
```

[![dotbackup-managed](https://img.shields.io/badge/dotbackup-managed-blue)](https://github.com/jaxvanyang/dotbackup.rs)
