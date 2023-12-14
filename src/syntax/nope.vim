" INSTALL:
" put this file in ~/.vim/syntax/nope.vim
" then add this to your .vimrc
"
"     autocmd BufNewFile,BufRead *.nope setlocal ft=nope
"
"


" Quit when a syntax file was already loaded.
if exists('b:current_syntax') | finish|  endif

syntax keyword letsetvar let set var do
syntax keyword cond if else
syntax keyword repeat while break break_as continue loop

syntax keyword stdlib random rand100 flip_coin d4 d6 d8 d10 d12 d20 d100
syntax keyword stdlib num print echo neg return not bool floor ceil abs acos acosh decr incr sin sinh asin asinh cos cosh tan tanh atan atanh inv log2 log10 ln1p ln exp expm1 sqrt cbrt round fround trunc sign str upper lower trim shh bitstr read_text is_even is_odd
syntax keyword stdlib add sub le leq ge geq eq aeq neq naeq max min mult div pow atan2 modulo join_paths write_text from_unit to_unit
syntax keyword stdlib replace

syntax match comment "\v#.*$"

" FIXME: not working as expected ...
" syntax match op "\v\="
" syntax match op "\v\!\="
" syntax match op "\v\<\="
" syntax match op "\v\>\="
" syntax match op "\v\+\-\="
" syntax match op "\v\!\+\-\="
" syntax match op "\v\*\*"
" syntax match op "\v&"
" syntax match op "\v||"
" syntax match op "\v\?\?"
" syntax match op "\v\*\:"
" syntax match op "\v\~\<\<"
" syntax match op "\v\~\>\>\>"
" syntax match op "\v\~\&"
" syntax match op "\v\~|"
" syntax match op "\v\~\!"
" syntax match op "\v\~\^"
" syntax match op "\v\~\>\>"
" syntax match op "\v\~\+"
" syntax match op "\v\~\-"
" syntax match op "\v\~\*"
" syntax match op "\v\~/"
" syntax match op "\v\<"
" syntax match op "\v\>"
" syntax match op "\v\+"
" syntax match op "\v\-"
" syntax match op "\v\*"
" syntax match op "\v/"
" syntax match op "\v\!"
" syntax match op "\v\%"

syntax match op "[-!|&+<>=%/*:~^]" skipwhite skipempty

" stolen from: https://github.com/pangloss/vim-javascript/blob/master/syntax/javascript.vim
syntax match num /\c\<\%(\d\+\%(e[+-]\=\d\+\)\=\|0b[01]\+\|0o\o\+\|0x\%(\x\|_\)\+\)n\=\>/
syntax match num /\c\<\%(\d\+\.\d\+\|\d\+\.\|\.\d\+\)\%(e[+-]\=\d\+\)\=\>/
syntax keyword num NaN Inf 
syntax keyword const Pi E SQRT_2 SQRT_2PI LN_2 LN_10 LOG2_10 LOG2_E LOG10_2 LOG10_E PHI TAU EPSILON MAX_F64 MIN_F64 MAX_U32 MAX_I32 MIN_I32 MAX_U16 MAX_I16 MIN_I16 MAX_U8 MAX_I8 MIN_I8 MAX_INT MIN_INT
syntax keyword bool true false
syntax keyword null null
syntax keyword void void _
syntax match void "\v\(\)"

"FIXME: doesn't work ...
syntax keyword todo contained TODO FIXME XXX


syntax region str start=/\v"/ skip=/\v\\./ end=/\v"/
syntax region str start=/\v'/ skip=/\v\\./ end=/\v'/

highlight link letsetvar Keyword
highlight link op Operator
highlight link stdlib Function
highlight link comment Comment
highlight link str String
highlight link num Number
highlight link const Constant
highlight link float Float
highlight link bool Boolean
highlight link null Constant
highlight link void Constant
highlight link cond Conditional
highlight link repeat Keyword
highlight link todo TODO

let b:current_syntax = "nope"
