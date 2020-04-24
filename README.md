<img src="img/ldpl-rs.png" alt="LDPL + Rust" align="right">

# LDPL in Rust

> [LDPL][ldpl] is a powerful compiled programming language designed
> from the ground up to be excessively expressive, readable, fast
> and easy to learn. It mimics plain English, in the likeness of the
> good parts of older programming languages like COBOL, with the
> desire that it can be understood by anybody. LDPL was designed to
> run on Unix systems, including AMD-64 Linux, macOS, ARMv8 Linux,
> Android Termux and both Intel and PowerPC OS X (tested from Tiger
> 10.4 onwards). It even supports UTF-8 out of the box.

— The [Official LDPL Repository][ldpl-repo]

---

This is an experimental **LDPL 4.4** compiler written in Rust. Like
the official compiler, LDPL code is translated to C++ code and then
compiled into a standalone binary.

## Building It

LDPL-RS requires **cargo**, **Rust**, **git**, and **a C++ compiler**
to work. Once you have all those, installation is a breeze:

    git clone git://github.com/xvxx/ldpl-rs
    cd ldpl-rs
    cargo build --release

If that works, you've successfully built an `ldpl-rs` binary that you
can use to run any of the official LDPL examples:

    git clone -b 4.4 git://github.com/lartu/ldpl
    ./target/release/ldpl-rs examples/99bottles.ldpl

To take it with you, just copy `./target/release/ldpl-rs` to
`/usr/local/bin` or wherever.

[ldpl]: https://www.ldpl-lang.org/
[ldpl-repo]: https://www.ldpl-lang.org/
[pest]: https://pest.rs/
