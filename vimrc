set expandtab
set tabstop=4
set shiftwidth=4
set colorcolumn=80
set hlsearch

:highlight ExtraWhitespace ctermbg=red guibg=red
:match ExtraWhitespace /\s\+$/

call plug#begin()
Plug 'rust-lang/rust.vim'
call plug#end()
