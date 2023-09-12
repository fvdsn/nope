#!/usr/bin/env python

import sys
import random


def rand_instr2(a, b):
    return [random.choice(["add", "sub", "mult"]), a, b]


def rand_instr1(a):
    return [random.choice(["neg"]), a]


def rand_no_exp():
    while True:
        r = random.random()
        if "e" not in str(r):
            return r


def rand_val():
    if random.randint(0, 1):
        return random.randint(-1000, 1000)
    else:
        return rand_no_exp()


def rand_expr(max_rec=10):
    r = random.randint(0, max_rec)
    if r == 0 or max_rec <= 0:
        return rand_val()
    elif r <= 1:
        return rand_instr1(rand_expr(max_rec - 1))
    else:
        return rand_instr2(rand_expr(max_rec - 1), rand_expr(max_rec - 1))


def print_expr(expr, indent=0, newline=True):
    if newline:
        spacing = " " * indent
    else:
        spacing = " "

    if type(expr) == int or type(expr) == float:
        print(f"{spacing}{expr}")
    elif len(expr) == 2:
        print(f"{spacing}{expr[0]}", end="")
        print_expr(expr[1], indent + 1, False)
    else:
        print(f"{spacing}{expr[0]}")
        for i in range(1, len(expr)):
            print_expr(expr[i], indent + 1, True)


max_rec = int(sys.argv[1])

print("print")
print_expr(rand_expr(max_rec))
