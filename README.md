# coil
a database engine
## Example
```
coil> create table customers [name: text, id: number]
coil> put ["james", 0xA] in customers
coil> get * from customers
```
## Grammar
```
query        -> create_query | get_query | put_query | update_query | delete_query
create_query -> "create" ( "database" identifier | "table" "[" ( identifier ":" field_type ","? )+ "]" )
field_type   -> "text" | "number"
get_query    -> "get" ( "*" | ( identifier ","? )+ ) "from" identifier ( "where" or )?
put_query    -> "put" "[" ( literal ","? )+ "]" "in" identifier
update_query -> "update" ( "[" identifier ":" literal ","? "]" )+ ( "where" or )? "in" identifier
delete_query -> "delete" "[" ( identifier ","? )+ "]" ( "from" identifier )?
or           -> and ( "or" and )*
and          -> equality ( "and" equality )*
equality     -> comparison ( ( "=" | "!=" ) comparison )*
comparison   -> term ( ( ">" | ">=" | "<" | "<=" ) term )*
term         -> factor ( ( "-" | "+" ) factor )*
factor       -> unary ( ( "/" | "*" | "**" | "%" ) unary )*
unary        -> ( "!" | "-" | "+" )? unary
              | primary
literal      -> number | string
primary      -> literal | identifier | "none"
              | "(" get_query ")" ;
```
### Notes
- The "from" keyword in `get_query` is unnecessary, as the `in` keyword serves the same purpose, but I like it 🤷‍♀️.
- The brackets `[]` around values are unnecessary, but I like them 🤷‍♀️.