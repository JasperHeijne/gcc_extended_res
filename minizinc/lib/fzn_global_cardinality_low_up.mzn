predicate fzn_global_cardinality_low_up(array[int] of var int: x,
                                       array[int] of int: cover,
                                       array[int] of int: lbound,
                                       array[int] of int: ubound)
 = pumpkin_gcc(x, cover, lbound, ubound);

predicate pumpkin_gcc(array[int] of var int: x,
                                        array[int] of int: cover,
                                        array[int] of int: lbound,
                                        array[int] of int: ubound);

predicate pumpkin_gcc_imp(array[int] of var int: x,
                                        array[int] of int: cover,
                                        array[int] of int: lbound,
                                        array[int] of int: ubound,
                                        var bool: b);