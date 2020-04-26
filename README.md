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
3. Pass all LDPL 4.4 [tests].
4. Compile popular LDPL 4.4 [projects].

| **Status** | **Goal**      | **Comments** |
| ---------- | ------------- | ------------ |
| 👷         | `examples/`   | Active       |
| 🚧         | `ldpltests`   | Planned      |
| 🚧         | `--flags`     | Planned      |
| 🚧         | `GILD`        | Planned      |
| 🚧         | `Lute`        | Planned      |
| 🚧         | `ldpl-socket` | Planned      |

## Statements

Tracking what we have left to do, based on the docs:

- [ ] flag "-O3"
- [ ] flag linux "-O3"
- [ ] extension "something.cpp"
- [ ] include "file.ldpl"
- [ ] DATA:
  - [ ] var IS type
    - [x] support LDPL var names
    - [x] LDPL var name => C++ var name
    - [x] NUMBER
    - [x] TEXT
    - [x] NUMBER LIST
    - [x] NUMBER MAP
    - [x] TEXT LIST
    - [x] TEXT MAP
    - [ ] MAP OF ...
    - [ ] LIST OF ...
  - [x] Predeclared variables
    - [x] ARGV IS TEXT LIST
    - [x] ERRORTEXT IS TEXT
    - [x] ERRORCODE IS NUMBER
- [ ] PROCEDURE:
  - [ ] NUMBER literals
    - [ ] regular
    - [ ] decimal
    - [ ] negative
    - [ ] +10 isn't valid
  - [ ] TEXT literals
    - [ ] TEXT escape codes
      - [ ] \a = alert (bell)
      - [ ] \b = backspace
      - [ ] \t = horizontal tab
      - [ ] \n = newline / line feed
      - [ ] \v = vertical tab
      - [ ] \f = form feed
      - [ ] \r = carriage return
      - [ ] \e = non-standard GCC escape
      - [ ] \0 = null byte
      - [ ] \\ = \ character
      - [ ] \" = " character
  - [ ] LIST lookup a:1
  - [ ] MAP lookup b:"name"
  - [ ] SUB-PROCEDURE
    - [ ] SUB syntax
    - [ ] END SUB / END SUB-PROCEDURE syntax
    - [ ] PARAMETERS: section
    - [ ] LOCAL DATA: section
    - [ ] PROCEDURE: section
- [ ] CONTROL FLOW
  - [x] STORE _ IN _
  - [x] IF _ IS _ THEN
  - [x] ELSE IF _ IS _ THEN
  - [x] WHILE _ IS _ DO
  - [ ] FOR _ FROM _ TO _ STEP _ DO
  - [ ] FOR EACH _ IN _ DO
  - [ ] BREAK
  - [ ] CONTINUE
  - [x] CALL \_
    - [x] CALL \_ WITH ...
  - [ ] RETURN
  - [ ] EXIT
  - [ ] WAIT \_ MILLISECONDS
  - [ ] GOTO and LABEL
  - [ ] CREATE STATEMENT _ EXECUTING _
  - [ ] CALL EXTERNAL \_
- [ ] ARITHMETIC
  - [x] IN _ SOLVE _
  - [x] FLOOR
  - [ ] CEIL
  - [x] FLOOR _ IN _
  - [ ] CEIL _ IN _
  - [x] MODULO _ BY _ IN \_
  - [ ] GET RANDOM IN \_
  - [ ] RAISE _ TO THE _ IN \_
  - [ ] LOG _ IN _
  - [ ] SIN _ IN _
  - [ ] COS _ IN _
  - [ ] TAN _ IN _
- [ ] TEXT
  - [ ] IN _ JOIN _
  - [ ] REPLACE _ FROM _ WITH _ IN _
  - [ ] SPLIT _ BY _ IN \_
  - [ ] GET CHARACTER AT _ FROM _ IN \_
  - [ ] GET LENGTH OF _ IN _
  - [ ] GET ASCII CHARACTER _ IN _
  - [ ] GET CHARACTER CODE OF _ IN _
  - [ ] STORE QUOTE _ IN _
  - [ ] GET INDEX OF _ FROM _ IN \_
  - [ ] COUNT _ FROM _ IN \_
  - [ ] SUBSTRING _ FROM _ LENGTH _ IN _
  - [x] TRIM _ IN _
- [ ] LIST
  - [ ] PUSH _ TO _
  - [ ] CLEAR
  - [ ] COPY _ TO _
  - [ ] GET LENGTH OF _ IN _
  - [ ] DELETE LAST ELEMENT OF \_
- [ ] MAP
  - [ ] CLEAR
  - [ ] COPY _ TO _
  - [ ] GET KEY COUNT OF _ IN _
  - [ ] GET KEYS OF _ IN _
- [ ] IO
  - [x] DISPLAY
  - [x] ACCEPT \_
  - [x] ACCEPT \_ UNTIL EOF
  - [x] EXECUTE \_
  - [x] EXECUTE _ AND STORE OUTPUT IN _
  - [x] EXECUTE _ AND STORE EXIT CODE IN _
  - [ ] LOAD FILE _ IN _
  - [ ] WRITE _ TO FILE _
  - [ ] APPEND _ TO FILE _
- [ ] C++ EXTENSIONS
  - [ ] var IS EXTERNAL type
  - [ ] CALL EXTERNAL
  - [ ] EXTERNAL SUB(-PROCEDURE)

[ldpl]: https://www.ldpl-lang.org/
[ldpl-repo]: https://www.ldpl-lang.org/
[ldpl-docs]: http://docs.ldpl-lang.org/
[pest]: https://pest.rs/
[rustup]: http://rustup.rs/
[git]: https://git-scm.com/book/en/v2/Getting-Started-Installing-Git
[cpp-compiler]: https://gcc.gnu.org/install/
[tests]: https://github.com/Lartu/ldpltest
[projects]: https://www.ldpl-lang.org/projects.html
[gild]: https://github.com/xvxx/gild
[lute]: https://github.com/lartu/lute
[ldpl-socket]: https://github.com/xvxx/ldpl-socket
