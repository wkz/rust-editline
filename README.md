rust-editline
=============

Rust bindings for the [editline] library.

Motivation
----------

editline provides line editing features similar to FSF's readline
library. Like readline it also allows you to plug in your own
completion backend. This makes it possible to create custom
interactive CLIs such as shells or configuration interfaces.

Here is a quick summary of why one might be interested in editline
instead of readline:

| Library  | Dependencies | Linking with non-GPL code |
|----------|--------------|---------------------------|
| readline | ncurses      | No                        |
| editline | None         | Yes                       |

Example
-------

See `examples/cli.rs`.

License
-------

ISC

[editline]: https://github.com/troglobit/editline
 