# The Nope Script

> This is project is currently at a very very early stage

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


You can add a unit at the end of a number declaration, this will convert its value to the default unit (internal standard). 

```
print add 5kg 2T  # this will print 2005
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
