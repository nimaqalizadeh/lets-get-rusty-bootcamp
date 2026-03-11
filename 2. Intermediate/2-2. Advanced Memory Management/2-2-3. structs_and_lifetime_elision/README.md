# Structs and lifetime elision

when storing references on structs, we must add generic lifetime annotations. By doing this, we are telling the borrow checker that the struct instance _cannot outlive_ the data it references

1. Each parameter that is a reference gets its own lifetime parameter.
2. If there is exactly one input lifetime parameter, that lifetime
   is assigned to all output lifetime parameters.
3. If there are multiple input lifetime parameters, but one o f them is
   &self or &mut self, the lifetime of self is assigned to all output
   lifetime parameters.

we don't need
