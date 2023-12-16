# The Nope Script

Nope is a scripting programming language, optimised for small and fun programs

### Nope is
 - Fun
 - Expression based
 - Garbage collected
 - Imperative with a sprinkle of functional
 - Focused on solving practical problems
 - Separates data from logic
 - Can represent JSON and XML natively
 - A single binary
 - Built with Rust
 - Will include a fully featured stdlib
 - Will have an eventloop concurrency model

![a screenshot of nope, see samples/fizzbuzz2.nope](samples/fizzbuzz_screenshot.png)

## Run and Install

Install rust, clone the repository and then

> cargo install --path .

You can then launch the interactive repl with

> nope

Or run a script with

> nope myscript.nope


## Roadmap

> This is project is currently at a very very early stage
> - [x] evaluate expressions
> - [x] global, local variables and scopes
> - [x] constant & operators
> - [x] math stdlib
> - [x] loops
> - [ ] functions
> - [ ] dictionaries and arrays
> - [ ] garbage collector
> - [ ] errors, try catch
> - [ ] string stdlib
> - [ ] parsing & serialization
> - [ ] utils stdlib
> - [ ] resource type
> - [ ] os stdlib
> - [ ] unit tests
> - [ ] v1.0
> - [ ] async
> - [ ] async stdlib
> - [ ] v2.0
> - [ ] web server
> - [ ] v3.0
> - [ ] modules & packages
> - [ ] v4.0
> - [ ] perf


## Nope Basics

### Basic types

The following table should give you an idea of the basic types and allowed data models. Keep in mind arrays and objects are not
implemented at the moment

``` 
[
    null: null,
    void: [void _ ()]
    bool: [true false]
    num:  [0 1 99 3.14 -1_000_000 0xdead 0b10101 NaN Inf PI]
    string: ['foo' "bar" ~hello]
    arrays: [1 2 true null []]
    dicts:  {key:32 val:99}
    mixed:  ['foo' key:'value']
]
```
All data in nope is a combination of the above types and structures.

Note that numbers are 64bit floats, and the strings are unicode.

### Basic code structure

Let's look at basic nope program.

```
let x = d6() + d6()
var winning = false

if x >= 10 (
    set winning = true
    print 'haha, yes! :)'
) else (
    print 'oh no :('
)

```

In the first line, we define a variable called `x`, with an initial value defined
by an expression which calls two functions 'd6' and adds the results. The variable
'x' is available in all subsequent expressions in the current scope. Since it
is defined with `let`, it cannot later be changed with `set`.

In the second line we declare another variable, but this time it can be modified.

We then evaluate a classic if-else conditional. Note that unlike other languages,
the code blocks are not defined by curly braces `{`/`}` but by parenthesis `(`/`)`. 
The reason is that nope is an expression based language, so every nope construct is
an expression that produces a value.

The following example puts those principles in action

```
let res = if flip_coin(), "heads" else "tails"
```

The `if` returns a value, which depending on the result of `flip_coin()` will be
either the string "heads" or "tails". 

You can surround expressions with parenthesis, and you can also surround multiple
expressions with parenthesis. In the later case, all expressions are evaluated in
order, but only the last expression produces a value.

This is what we've used to both change the variable `winning` with the `set winning = true`
expression, but also to print a message.

Note as well that the print expression `print "hey"` ommits parenthesis around its argument
This is not a special case of the print function. In nope, all parenthesis and commas for
functions arguments are optional.

```
let clamped_0_100 = max 0 min value 100
```

This is also applies to functions without parameters

```
let score = d6 + d6 + d6
```

Note that function application has the highest precedence so `print 1 + 1` prints `1`
since the addition is performed afterwards. This is not always what you want. One way
is to put the parenthesis like this, `print(1 + 1)`. Or like this `print (1+1)`. Note
that the space between the function name and the first parenthesis is meaningful. Without
space, the full list of arguments is expected, while with the space, it's just parenthesis
around a single argument; `max(1,2)` / `max (1) (2)`

Another way is to use the left arrow `<-` operator. This operator simply evaluates the complete expression
on the right. So you can do your print like this `print <- 1 + 1`.

By convention, all functions in nope take the data they operate on as the last argument.

```
let txt = read_text 'file.txt'
let txt2 = upper txt
write_txt 'file2.txt' txt2
```

With the help of the left arrow `<-` operator you could have instead written the program like this:

```
write_txt 'file2.txt' <- upper <- read_text 'file.txt'
```

The right arrow `->` operator (not yet implemented) does the same but the other way around, and allows to write the following

```
read_text 'file.txt' -> upper -> write_txt 'file2.txt'
```



### Operators

#### Equality `==`, `!=`

Check that the operands are of the same type, and if that's the case compare their values or references

#### Comparison `<`, `>`, `<=`, `>=`

First converts the number to their number equivalent (NaN for strings and arrays) then compares

The `+-=`and `!+-=` compare numbers for equality up to `EPSILON` precision

#### Arithmetic `+`, `-`, `*`, `/`, `%`, `**`, 

The usual math operations. Operands are converted to their number equivalent (NaN for string)

Addition `+` of a string will perform string concatenation, and convert the other operand to string

#### Integer & Bitwise Arithmetic

All the arithmetic operators prefixed with `~` will first convert operands to int32 then perform the integer arithmetic operation.

For example `10 ~/ 4` is `2`

The bitwise operators `~!`, `~|`, `~&`, `~^`, `~<<`, `~>>`, `~>>>`
convert the operands to int32 then perform the bitwise operations


#### Boolean `!`, `||`, `&&`

The boolean equivalent of an operand is
 - `false` if `null`, `void` or `0`
 - `true` otherwise

The operators
- `!` negates the boolean equivalent
- `&&` returns the first operand if falsy, otherwise the second
- `||` returns the first operand if truthy, otherwise the second

#### Default `??`

Will return the second operand if the first is `null` or `void` otherwise the first

#### Repeat `*:`

The repeat operator will evaluate and sum the second operand N times, where N is the first operand. 

For example, this will throw the dice 3 times

```
print(3 *: d6)
```

Note that the first operand is first converted to a number. If the number is zero or negative, we return `void`.

### Arithmetic Functions and constants

Nope supports the following arithmetic functions

- `floor`, `ceil`, `abs`, `acos`, `acosh`, `sin`, `sinh`, `asin`, `asinh`, `cos`, `cosh`, `tan`, `tanh`, `atan`, `atanh`, `inv` `log2`, `log10`, `ln`, `ln1p`, `exp`, `expm1`, `sqrt`, `cbrt`, `round`, `fround`, `trunc`, `sign`, `max`, `min`, `pow`, `atan2`

And the following arithemtic constants

- `NaN`, `Inf`, `PI`, `E`, `SQRT_2`, `SQRT_2PI`, `LN_2`, `LN_10`, `LOG2_10`, `LOG2_E`, `LOG10_2`, `LOG10_E`, `PHI`, `TAU`, `EPISLON`, `MAX|MIN_INT` `MAX|MIN_F64`, `MAX|MIN_I32|16|8`, `MAX_U32|16|8`


### Units







- - - 


The two keywords `var` and `let` allow you to define variables. Only the variables
defined with 'var' can be later changed.

```
let foo = 32
var bar = false
```

Variables are syntaxically scoped; they are only available in the expression following
the declaration.





Nope is lisp without the parenthesis, a programming language optimised for fun, small useful scripts, and repl oriented development


Nope is an expression based language. Every function has a fixed number of arguments and is immediately applied upon referencement

```
print add 2 3 # this will print 5
```

You declare variables with `let varname initialvalue expr`

```
let place "World"
print concat "Hello, " place
```

Functions have the form `|arg1 arg2 ... argn| expr`

```
let add42 |arg| add 42 arg
print add42 99
```

### Numbers

Numbers are just 64bit floats:

```
10 -3.1415 1_000_000 NaN Inf
```


You can add a unit at the end of a number declaration, this will convert its value to the default unit (SI). 

```
print add 5kg 2T  # this will print 2005
```

This is also how you access the usual mathematical constants

```
let area mult 10cm cos 0.5pi
```

### Booleans

Boolean logic is pretty straigthforward, `true`, `false`, boolean functions `and a b`, `or a b`, `not a`.

the `ife cond expr1 expr2` expression (if/else) evaluates the first or the second expression depending if the condition is truthy or not

```
ife and true not false
    print "yes"
    print "no"
```

### Strings

Strings can be either single quoted or double quoted and are multilines by default.
Any sequence of characters that starts with a `-` and is not a number is also interpreted as a string (without the dash)

```
"foo" 'foo' -foo
```

This is makes it cleaner to use string as enums

```
let data parse -json readtxt -utf8 "./file.json"
```

Strings that respect well known formats are automatically parsed and the parsed results are available as fields

```
print ext."/path/to/file.png" # png
```

### Array and Dictionaries

Arrays and dictionaries are using a single representation. 

```
let person [name:'Francois' age:32y childrens:['Nicolas' 'Gérladine']]
```

You can mix and match keyed and indexed values in the same array

```
let xml [-div id:123 class:'foo bar' "Click on this" [-a href:'#' "link"]]
```

#### Matrices

You can also make two dimensional arrays by separating rows with `,` this is used for linear algebra.

```
let mat3 [
    1 2 3,
    4 5 6,
    7 8 9,
]

print rowc mat3   # 3
print colc mat3   # 3
print len mat3    # 9
```

The commas can only divide the array with consistently sized rows. The array usually behaves as a regular
array, iteration works in one dimension. But linear algebra functions can make use of `colc` and `rowc`.

### `null` and `void` / `_`

Nope has null and void as two separate concepts used to signify the lack of value. `null` means the intentional absence of a value. For example a valid field who's value hasn't been set. `void` signifies the logical absence of a value. for example a function that just has side effects but no meaningful value to return. Or access to a key that does not exist in an array.

`void` can also be written as `_`, that keyword can also be used as a function parameter identifier to signify that you are not interested in the parameter value `iter range 0 100 |_| print 'repeat'`

### Accessing data in arrays

you can access fields using dot notation and array indexing, however the association goes leftward. Integer keys access array elements, other keys are converted to string and access dictionary values

```
let arr = [1 2 3 4 5]
let first [0]arr
let last  [-1]arr
let notfound [100]arr # void
let l len arr # 5

do iter arr |val| print val
do set [1]arr 42

let even [|v| is-even v]arr
let double map |v| mult 2 v arr

let dict [key:'value' key2:[123]]

let a key.dict
let b ['key'].dict
let c contains 'key' dict
let l count dict # 2

do iter keys dict |key val| print join '' ['key:' key ' value:' [key]dict]
```

### Errors

you can mark a value as an error with `!`

```
let get-strict |key dict|
    ife not contains key dict
        ret !-key-not-found
    ret [key]dict
```

you can then provide a default value in case of errors with `try |default value|`

```
let age try 18 get-strict -age person
```

or make the program crash with a message with `expect |errmsg value|`
```
let age expect "Please provide your age" get-strict -age person
```

if you want to capture an error that comes from a function argument you need to 
capture the errors of that argument by prefixing it with `!`, otherwise the function
immediately returns with its first error argument

```
let log-errors |!err|
    do if is-error err
        print err
    ret err

let print-hello |place|
    # this body is never executed if place is an error
    print join '' ["Hello, " place "!"]

log-errors print-hello get-strict 'usa' countries
```

### Imperative programming

What if you want to do multiple prints in a row ? You can use and chains the  `do |expr1 expr2|` 
expression. This evaluates both expression and returns the value of the second. You may use end to
return `void` at the end of the chain

```
let print-user |user|
    do print name.user
    do print email.user
    do print map |_| '*' password.user
    end
```

### Parenthesis

It is possible to add parenthesis around an expression. This has no effect on the code except for styling or error reporting when you
make syntax mistakes. An empty set of parenthesis is `void`

```
ife neq a b (
    print "a is different from b !"
) (
    print "a is equal to b"
)

print( eq void () ) # true

(add 3 4)
```

parenthesis can also close a do expression early which then implicitly returns void

```
def print-point |p| (
    do print x.p
    do print y.p
)
```

# Commas

It is possible to add commas between function arguments. If you choose to do so you must put a comma
between every argument of that function call. This is again an optional way to improve readability & ease debugging

```
print func-with-many-args -opt, add 2 3, and == x y <= x z
```

### Macros 

if you mark an argument of a function with `$` the argument will contain the rest of the line from that argument onwards as a raw source code string, with $values expanded to string interpolations of current variables

```
let ls |$args| lines stdout.call 'ls' args

iter ['./foo/' './bar'] |dir|
    print [0]ls -la $dir
```

You can also use multi lines macros, they are delimited by `[[[` `]]]`

```
let python |version $source|
    let py default !version-not-found [version][-v2.0:'/usr/bin/python2' -v3.0:'/usr/bin/python3']
    let tmp open-temp-file
    do write tmp source
    let res call py path.tmp
    ife neq 0 errno.res
        ret !stderr.res
    ret stdout.res


let world 'World'

print python -v3.0 print("Hello $world")

print python -v2.0 [[[
for fizzbuzz in range(51):
    if fizzbuzz % 3 == 0 and fizzbuzz % 5 == 0:
        print("fizzbuzz")
    elif fizzbuzz % 3 == 0:
        print("fizz")
    elif fizzbuzz % 5 == 0:
        print("buzz")
    else:
        print(fizzbuzz)
]]]
```

### Configuration

Nope's whole configuration is always given as arguments to the nope executable. If this is provided in the first line of the script, it will be taken into account.

```
#!/usr/bin/nope --version=0.1

print "Hello World!"
```

