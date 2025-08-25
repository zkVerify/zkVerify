%builtins output

from starkware.cairo.common.serialize import serialize_word

func main{output_ptr: felt*}():
    // Simple computation: x^2 + y^2 = z^2 (Pythagorean triple)
    let x = 3
    let y = 4
    let z = 5
    
    // Verify the computation
    assert x * x + y * y == z * z
    
    // Output the result
    serialize_word(x)
    serialize_word(y)
    serialize_word(z)
    
    return()
end
