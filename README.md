# Sprachli

## Grammar

```
program: declaration* EOF;

declaration:
	use |
	fn |
	struct |
	mixin |
	impl;

use: "pub"? "use" path ";";

fn: "pub"? "fn" IDENTIFIER "(" identifierList ")" block;

struct: "pub"? "struct" (
	"(" identifierList ")" ";" |
	"{" identifierList "}"
);

mixin: "pub"? "mixin" mixinOrImplBody;

impl: "impl" mixinOrImplBody;

mixinOrImplBody:
	identifier
	(":" (path ("," path) ","?)?)?
	("{" declaration* "}" | ";");

identifierList: (identifier ("," identifier) ","?)?;

path: (
	("super") ("::" identifier)+ |
	identifier ("::" identifier)*
) ("as" identifier)?;

block: "{" statement* expression? "}";

statement:
	declaration |
	expression ";";

expression: NUMBER;
```
