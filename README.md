# Nope

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
