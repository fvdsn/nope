# The Nope Programming Language

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

Numbers are just 64bit floats:

```
10 -3.1415 1_000_000 NaN Inf
```


You can add a unit at the end of a number declaration, this will convert its value to the default unit (internal standard). 

```
add 5kg 2T  # this will print 2005
```

Boolean logic is pretty straigthforward, `true`, `false`, boolean functions `and a b`, `or a b`, `not a`.

```
ife and true not false
    print "yes"
    print "no"
```

### Null & Void







## Examples

```
let name value expression

let a 42
let b 99
add a b

let fibo |n|
    ife leq n 0
        0
    ife leq n 1 
	1
        add fib sub n 1
            fib sub n 2

eq 'foo' -foo

let f open -rw './file.txt' 
map lines read -utf8 f |line|
    print line

[3.14 true false null void 'string' [] key:'value']

print serialise -xml [-html 
  [-body 
     [-a href:'google.com' 'click here']
  ]
]

let sort |array cmp:2|
	try 0 cmp [i]arr [decr i]arr
	...

let sorted sort dup [3 1 4] @leq

!'error' 


let x sin 90deg

wait 1min


let obj [x:42] 
let foo get -x obj

ext.'/home/fred/file.bin' 

let s makestream []
thread defer close s while ||
    let result curl -get 'https://google.com?q=dogs'
    ife result
	push s result
    break

thread map s |dog| print dog 

let fakeyear year.birthday.[0]parse -jsonlines stdout.call[-fakeidgen json:true num:42]

let curl $|args|
    let res call[-curl args]
    ife stderr.res
	!stderr.res
    stdout.res

let alias |name|:$1
    $|args| let res call[name args] ife stderr.res !stderr.res stdout.res

let curl alias 'curl'

let count 1
let fakename name.[0]parse --jsonlines curl -GET https://ddm.io/api/fakeidgen/?num=$count
```

## Error messages

```
    Computer says no
    WRONG!!!
    ? WHY ?
    Unknown Error: Failed(0)
    Segmentation Fault
    I am a teapot
    Try Again ? (Y/n)
    LOL
    lol, lmao even
    ok
    :facepalm:
    I am sorry Dave, I can't do this 
``` 
