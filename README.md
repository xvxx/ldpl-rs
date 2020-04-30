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
compiled into a standalone binary. Generated code should be 100%
compatible with the official compiler, meaning LDPL-RS should work
just fine with regular LDPL extensions.

## Installation

LDPL-RS requires [**cargo**][rustup] to install and a [**C++
compiler**][cpp-compiler] to use.

Once you've got them both, installation is a breeze:

    cargo install ldpl

You should now have an **ldpl-rs** binary that you can use to compile
simple LDPL 4.4 programs, like any of the official examples:

    git clone -b 4.4 git://github.com/lartu/ldpl
    ldpl-rs ldpl/examples/99bottles.ldpl

Note the difference between the Crate name and binary. This is to
avoid collision with the official LDPL compiler.

## Building It

Building from source requires **make** and [**cargo**][rustup].
LDPL-RS also requires a [**C++ compiler**][cpp-compiler] to build LDPL
programs, so you'll need one installed even after building the
`ldpl-rs` binary.

Once you've got all that, clone and compile the project:

    git clone git://github.com/xvxx/ldpl-rs
    cd ldpl-rs
    make

If that works, you've successfully built an `ldpl-rs` binary that you
can use to compile any of the official LDPL examples:

    git clone -b 4.4 git://github.com/lartu/ldpl
    ./ldpl-rs ldpl/examples/99bottles.ldpl

You can also use the "run" command to build and run a file in one go:

    ./ldpl-rs run ldpl/examples/99bottles.ldpl
    99 bottles of beer on the wall...

To take it with you, just copy `ldpl-rs` to `/usr/local/bin` or
something else in your `$PATH`.

## Status

This project is in its infancy, but can compile simple LDPL programs.
It supports all LDPL 4.4 statements, including C++ extensions, and
can compile and run all the examples that shipped with LDPL 4.4. It
can also compile [Gild][gild] and [ldpl-todo].

It passes 11 of 12 of the [official LDPL tests][ldpltest].

However, these features are currently unsupported (but coming soon):

- [ ] nested collections (NUMBER LIST LIST LIST)
- [ ] OF syntax (LIST OF NUMBERS)

To run the tests, clone this project (instructions above) and run:

    make test

## [LDPLTest][ldpltest] Pass/Fail Status

| **Test** | **Status** | **Failure Reason** |
| -------- | ---------- | ------------------ |
| basicar  | ✅         |                    |
| basictx  | ✅         |                    |
| conflow  | ✅         |                    |
| exec     | ✅         |                    |
| explode  | ✅         |                    |
| fibo     | ✅         |                    |
| file     | ✅         |                    |
| list     | ✅         |                    |
| of       | ❌         | OF syntax          |
| quine    | ✅         |                    |
| sqrt     | ✅         |                    |
| vector   | ✅         |                    |

## [LDPL Examples][examples] Pass/Fail Status

| **Example**         | **Status** | **Failure Reason** |
| ------------------- | ---------- | ------------------ |
| 99bottles.ldpl      | ✅         |                    |
| absolutevalue.ldpl  | ✅         |                    |
| arguments.ldpl      | ✅         |                    |
| bellman-ford.ldpl   | ✅         |                    |
| brainfuck.ldpl      | ✅         |                    |
| disancount.ldpl     | ✅         |                    |
| euler.ldpl          | ✅         |                    |
| explode.ldpl        | ✅         |                    |
| factorial.ldpl      | ✅         |                    |
| fibonacci.ldpl      | ✅         |                    |
| floyd-warshall.ldpl | ✅         |                    |
| helloworld.ldpl     | ✅         |                    |
| leapyear.ldpl       | ✅         |                    |
| loop_counter.ldpl   | ✅         |                    |
| oddornot.ldpl       | ✅         |                    |
| quine.ldpl          | ✅         |                    |
| sqrt.ldpl           | ✅         |                    |
| strcmp-demo.ldpl    | ✅         |                    |

## License

The LDPL-RS Compiler is distributed under the Apache 2.0 License, same
as the official LDPL compiler. All LDPL Dinosaur logos where created
by [Lartu](https://github.com/Lartu) and are released under a Creative
Commons Attribution 4.0 International (CC BY 4.0) license.

Portions of LDPL-RS are copied directly from LDPL. Thank you to the
LDPL community for all their contributions!

[ldpl]: https://www.ldpl-lang.org/
[ldpl-repo]: https://www.ldpl-lang.org/
[ldpl-docs]: http://docs.ldpl-lang.org/
[pest]: https://pest.rs/
[rustup]: http://rustup.rs/
[git]: https://git-scm.com/book/en/v2/Getting-Started-Installing-Git
[cpp-compiler]: https://gcc.gnu.org/install/
[ldpltest]: https://github.com/Lartu/ldpltest
[projects]: https://www.ldpl-lang.org/projects.html
[gild]: https://github.com/xvxx/gild
[ldpl-todo]: https://github.com/xvxx/ldpl-todo
[lute]: https://github.com/lartu/lute
[ldpl-socket]: https://github.com/xvxx/ldpl-socket
[examples]: https://github.com/Lartu/ldpl/tree/4.4/examples
