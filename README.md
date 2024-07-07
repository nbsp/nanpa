# nanpa
a language-agnostic release manager

> [!NOTE]
> nanpa is a command-line tool. for continuous integration, [ilo](https://github.com/nbsp/ilo)
> is a GitHub bot and workflow action that uses nanpa to automate package updates.

## usage

refer to **nanpa**(1) for CLI usage notes and **nanparc**(5) for information on configuration.

## building
```sh
$ cargo build --release

# or directly to your bin directory
$ cargo install --path .
```

## support
if you need help, or you think you've found a bug, send a [plain text 
email](https://useplaintext.email) to [the mailing list](mailto:~nbsp/public-inbox@lists.sr.ht).
the issue tracker is for *confirmed bugs only*; unconfirmed issues and general support requests will
be closed.

## contributing
open pull requests, or send patches to [the mailing list](https://lists.sr.ht/~nbsp/public-inbox).

prefix patches with "`[PATCH nanpa]`". see [the guide to `git send-email`](https://git-send-email.io)
if this is your first time using sourcehut.

## license
nanpa is licensed under the MIT license. refer to [the license](LICENSE) for details.
