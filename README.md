# ldpl-rs

An experimental [LDPL 4.4][ldpl] compiler written in Rust.

Uses [pest] for parsing.

SECTIONS:

- [ ] DATA:
- [ ] PROCEDURE:

DATA:

- [ ] INCLUDE "file"
- [ ] x IS TEXT
- [ ] x IS NUMBER
- [ ] LIST
  - [ ] x IS NUMBER LIST
  - [ ] x IS TEXT LIST
  - [ ] x IS LIST OF TEXT
  - [ ] x IS LIST OF LISTS OF TEXT...
- [ ] MAP
  - [ ] x IS TEXT MAP
  - [ ] x IS NUMBER MAP
  - [ ] x IS MAP OF TEXT MAPS...

SUB-PROCEDURE: - [ ] SUB-PROCEDURE name
PARAMETERS:
x IS var
LOCAL DATA:
y IS var
PROCEDURE:
code
END SUB

PROCEDURE:

## FLOW

[ldpl]: https://www.ldpl-lang.org/
[pest]: https://pest.rs/
