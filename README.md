# Serde bibtex

Thoughts:
- flexible parsing? handling comments which appear outside values, more lenient identifier names (allow "'"), @COMMENT parse until closed bracket } or until hit @, whatever happens later

Features:
- idempotence...
- guaranteed valid output...

## Some implementation details
### Parsing
The grammar used to parse `.bib` files is as follows.
The syntax is a [parsing expression grammar](https://en.wikipedia.org/wiki/Parsing_expression_grammar) (PEG) which follows the [pest grammar for PEGs](https://docs.rs/pest/latest/pest/).
```
ws = _{ (" " |  "\t" | "\n")* }
comment = _{ (!"@" ~ ANY)* }

bibtex = _{ SOI ~ comment ~ (command_or_entry ~ comment)* ~ EOI }
command_or_entry = _{ "@" ~ ws ~ (comment_command | preamble | string | entry) }

comment_command = _{ ^"comment" ~ &(" " | "\t" | "\n" | "{" | "(") }

preamble = { ^"preamble" ~ ws ~ ("{" ~ ws ~ value ~ ws ~ "}" | "(" ~ ws ~ value ~ ws ~ ")") }

string = { ^"string" ~ ws ~ ("{" ~ ws ~ string_body ~ ws ~ "}" | "(" ~ ws ~ string_body ~ ws ~ ")") }
string_body = _{ identifier ~ ws ~ "=" ~ ws ~ value }

entry = { identifier ~ ws ~ ("{" ~ ws ~ entry_key ~ fields? ~ ws ~ "}" | "(" ~ ws ~ entry_key ~ fields? ~ ws ~ ")") }

field_sep = _{ ws ~ "," ~ ws }
entry_key = { (!("{" | "}" | "(" | ")" | "," | " " | "\t" | "\n") ~ ANY)+ }
fields = _{ (field_sep ~ field)* ~ field_sep? }
field = { identifier ~ ws ~ "=" ~ ws ~ value }

value = { field_token ~ (ws ~ "#" ~ ws ~ field_token)* }
field_token = _{ ASCII_DIGIT+ | "{" ~ balanced* ~ "}" | "\"" ~ (!"\"" ~ balanced)* ~ "\"" }
balanced = _{ "{" ~ balanced* ~ "}" | (!("{" | "}") ~ ANY) }

identifier = { !ASCII_DIGIT ~ (!(" " | "\t" | "\\" | "#" | "%" | "'" | "(" | ")" | "," | "=" | "{" | "}") ~ ' '..'~')+ }
```
Note that BibTeX allows round brackets `()` inside an `entry_key`, but for example round brackets are not permitted by tools such as `biber`.
Moreover, one of goals of this crate is o allow round-trip parsing and emitting, and round brackets in an `entry_key` cannot be canonically normalized without changing the underlying data.
As a result, we choose to not support such keys.

### Emitting
Valid BibTeX has a reasonable amount of flexibility in terms of whitespace, brackets, capitalization, etc.
For example, the following code is valid BibTeX:
```
@string{A = "Author"}
@
 string{A0={One, } #
A }

    @ commENt {nothing

  @  arTIcle 

  (key,
    author = A0#
      " and Two, Author",
    year = 2014

    ,

                         
             journal               
               = "A journal",
    title = {An} # " example"
)

@  article  {key2 }
```
When emitting parsed objects, the BibTeX is normalized.
Parsing and then emitting the above string will yield
```bib
@article{key,
  author = {One, Author and Two, Author},
  year = {2014},
  journal = {A journal},
  title = {An example},
}

@article{key2,
}
```
Carefully inspecting the grammar, we can see that the emitted content is always valid and the original data is preserved, assuming that the content was initially parsed from a valid `.bib` file.
This would not be the case if we instead used quotation marks `"..."` in place of the brackets `{...}`.
