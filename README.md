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

## Building It

LDPL-RS requires **make**, [**cargo**][rustup], [**Rust**][rustup],
[**git**][git], and a [**C++ compiler**][cpp-compiler] to build. Once
you have all those, installation is a breeze:

    git clone git://github.com/xvxx/ldpl-rs
    cd ldpl-rs
    make

If that works, you've successfully built an `ldpl-rs` binary that you
can use to run any of the official LDPL examples:

    git clone -b 4.4 git://github.com/lartu/ldpl
    ldpl-rs examples/99bottles.ldpl

To take it with you, just copy `ldpl-rs` to `/usr/local/bin` or
something else in your `$PATH`.

## Status

This project is in its infancy. It's currently focused on implementing
all LDPL 4.4 statements, including C++ extension support. Once that's
done, we'll move onto these four, higher level goals:

1. Support the same `--flags` as the official compiler.
2. Compile all LDPL 4.4 `examples/`.
3. Pass all LDPL 4.4 [tests][ldpltest].
4. Compile popular LDPL 4.4 [projects].

| **Status** | **Goal**      | **Comments** |
| ---------- | ------------- | ------------ |
| 👷         | `examples/`   | In Progress  |
| 👷         | `ldpltests`   | In Progress  |
| 👷         | `--flags`     | Mostly done  |
| 🚧         | `GILD`        | Planned      |
| 🚧         | `Lute`        | Planned      |
| 🚧         | `ldpl-socket` | Planned      |

### [LDPLTest][ldpltest] Pass/Fail Status

| **Test** | **Status** | **Failure Reason**   |
| -------- | ---------- | -------------------- |
| basicar  | ✅         |                      |
| basictx  | ✅         | Type coercion        |
| conflow  | ❌         | CREATE STMT          |
| exec     | ✅         |                      |
| explode  | ✅         |                      |
| fibo     | ✅         |                      |
| file     | ✅         |                      |
| list     | ✅         | Type coercion        |
| of       | ❌         | OF syntax            |
| quine    | ✅         | Type coercion        |
| sqrt     | ✅         |                      |
| vector   | ✅         | Type coercion (keys) |

### [LDPL Examples][examples] Pass/Fail Status

| **Example**         | **Status** | **Failure Reason** |
| ------------------- | ---------- | ------------------ |
| 99bottles.ldpl      | ✅         |                    |
| absolutevalue.ldpl  | ✅         |                    |
| arguments.ldpl      | ✅         |                    |
| bellman-ford.ldpl   | ❌         | Infinity symbol    |
| brainfuck.ldpl      | ✅         |                    |
| disancount.ldpl     | ✅         |                    |
| euler.ldpl          | ✅         |                    |
| explode.ldpl        | ❌         | ACCEPT for text    |
| factorial.ldpl      | ✅         |                    |
| fibonacci.ldpl      | ✅         |                    |
| floyd-warshall.ldpl | ❌         | Infinity again     |
| helloworld.ldpl     | ✅         |                    |
| leapyear.ldpl       | ❌         | ELSE IF            |
| loop_counter.ldpl   | ✅         |                    |
| oddornot.ldpl       | ✅         |                    |
| quine.ldpl          | ✅         |                    |
| sqrt.ldpl           | ✅         |                    |
| strcmp-demo.ldpl    | ✅         |                    |

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
[lute]: https://github.com/lartu/lute
[ldpl-socket]: https://github.com/xvxx/ldpl-socket
[examples]: https://github.com/Lartu/ldpl/tree/4.4/examples
