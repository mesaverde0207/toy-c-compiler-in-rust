#!/bin/bash
assert() {
    expected="$1"
    input="$2"

    cargo run -- "$input" > tmp.s
    cc -o tmp tmp.s
    ./tmp
    actual="$?"

    if [ "$actual" = "$expected" ]; then
        echo "$input => $actual"
    else
        echo "$input => $expected expected, but got=$actual"
        exit 1
    fi

    rm tmp tmp.s
}

assert 0 "0;"
assert 42 "42;"
assert 21 "5+20-4;"
assert 41 " 12 + 34 -  5;"
assert 47 "5 + 6 * 7;"
assert 15 "5 * (9 - 6);"
assert 4 "(3 + 5) / 2;"
assert 6 "-2 * -3;"
assert 1 "-2 + 3;"
assert 2 "1 + -2 + 3;"
assert 1 "2 ==2;"
assert 1 "1 == 1;"
assert 0 "2 + 2 == 5;"
assert 0 "1 != 1;"
assert 1 "2 + 2 != 5;"
assert 1 "1 < 2;"
assert 0 "2 < 1;"
assert 0 "1 > 2;"
assert 1 "1 <= 2;"
assert 0 "2 <= 1;"
assert 0 "1 >= 2;"
assert 3 "1; 2; 3;"
assert 1 "a = 1; a;"
assert 3 "a = 1; b = 2; a+b;";
assert 2 "a = b = 1; a + b;"
assert 1 "foo = 1; foo;"
assert 6 "a_b = 2; c12 = 3; a_b*c12;"
assert 1 "_ab = 1; _ab;"
assert 1 "return 1; 2; 3;"
assert 2 "1; return 2; 3;"
assert 3 "1; 2; return 3;"
assert 1 "retur = 1; retur;"
assert 4 "{ 1; {2; 3;} return 4;}"
assert 3 "{ 1; {2; return 3;} 4;}"
assert 5 "{ ;;; return 5;}"
assert 2 "if (0) return 1; return 2;"
assert 2 "if (1-1) return 1; return 2;"
assert 2 "if (1) { 1; return 2;} return 3;"
assert 1 "if (1) return 1; return 2;"
assert 1 "if (2-1) return 1; return 2;"
assert 5 "if (0) { 1; 2; return 3; } else { 4; return 5; }"
assert 3 "if (1) { 1; 2; return 3; } else { 4; return 5; }"
assert 6 "if (1) { 1; 2; if (1) return 6; } else { 4; return 5; }"
assert 55 "i = 0; j = 0; for (i = 0; i <= 10; i = i + 1) j = i + j; return j;"
assert 3 "for (;;) return 3; return 5;"
assert 10 "i = 0; while (i < 10) { i = i+1; } return i;"
assert 3 "x = 3; return *&x;"
assert 3 "x = 3; y = &x; z = &y; return **z;"
assert 2 "x = 1; y = 2; return *(&x - 8);"
assert 1 "x = 1; y = 2; return *(&y + 8);"
assert 2 "x = 1; y = &x; *y = 2; return x;"
echo OK
