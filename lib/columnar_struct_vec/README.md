# Columnar Struct Vec

This library provides a macro making it easy to build Struct's of Vec's.
When building up Structs of Vecs from Green Button data, it's annoying to track which fields we've seen and which we haven't, to know
when we need to insert default values.

This abstracts away that problem.

### History

At one point, I made this library automatically generate wasm and arrow bindings, as well as provide nice iterators etc.
In retrospect, the complexity wasn't worth it, so I stripped this down as much as possible. I think this library should be
as minimal as possible.
