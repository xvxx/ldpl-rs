# ldpl-rs

An LDPL 4.4 compiler written in Rust.

TOKENS:

- [ ] NUMBER
- [ ] TEXT
- [ ] VAR
- [ ] LIST LOOKUP
- [ ] MAP LOOKUP

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

- [ ] STORE _ IN _
- [ ] IF _ IS _ THEN
- [ ] ELSE IF _ IS _ THEN
- [ ] WHILE _ IS _ DO
- [ ] FOR _ FROM _ TO _ STEP _ DO
- [ ] FOR EACH _ IN _ DO
- [ ] BREAK
- [ ] CONTINUE
- [ ] CALL SUB-PROCEDURE
- [ ] RETURN
- [ ] EXIT
- [ ] WAIT \_ MILLISECONDS
- [ ] GOTO and LABEL
- [ ] CREATE STATEMENT _ EXECUTING _
- [ ] CALL EXTERNAL \_
- [ ] IN _ CALL PARALLEL _
- [ ] WAIT FOR PARALLEL \_
- [ ] STOP PARALLEL \_

## MATH

- [ ] IN _ SOLVE _
- [ ] FLOOR
- [ ] CEIL
- [ ] FLOOR _ IN _
- [ ] CEIL _ IN _
- [ ] MODULO _ BY _ IN \_
- [ ] GET RANDOM IN \_
- [ ] RAISE _ TO THE _ IN \_
- [ ] LOG _ IN _
- [ ] SIN _ IN _
- [ ] COS _ IN _
- [ ] TAN _ IN _

## TEXT

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
- [ ] TRIM \_ IN

## LIST

- [ ] PUSH _ TO _
- [ ] CLEAR
- [ ] COPY _ TO _
- [ ] GET LENGTH OF _ IN _
- [ ] DELETE LAST ELEMENT OF \_

## MAP

- [ ] CLEAR
- [ ] COPY _ TO _
- [ ] GET KEY COUNT OF _ IN _
- [ ] GET KEYS OF _ IN _

## IO

- [ ] DISPLAY
- [ ] ACCEPT \_
- [ ] EXECUTE \_
- [ ] EXECUTE _ AND STORE OUTPUT IN _
- [ ] EXECUTE _ AND STORE EXIT CODE IN _
- [ ] ACCEPT \_ UNTIL EOF
- [ ] LOAD FILE _ IN _
- [ ] WRITE _ TO FILE _
- [ ] APPEND _ TO FILE _
