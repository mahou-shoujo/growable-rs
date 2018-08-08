var N = null;var searchIndex = {};
searchIndex["growable"]={"doc":"A growable, reusable box for Rust.","items":[[3,"GrowablePoolBuilder","growable","A customizable [`GrowablePool`] builder.",N,N],[3,"GrowablePool","","A pool of [`Growable`] objects. Unlike a typical Arena-based allocator it probably will not be able to decrease a memory fragmentation or provide some strong guarantees about frequency of allocations in your code but instead can be used to reduce the total amount of allocations in an amortized way by reusing the same memory to store different objects.",N,N],[3,"Growable","","A chunk of the heap memory that can be assigned with an arbitrary type.",N,N],[3,"Reusable","","A reusable box. It behaves just like the default [`Box`] (and it WILL free memory on drop) but it is also possible to free it manually, fetching a [`Growable`] back.",N,N],[5,"replace","","Replaces the value, dropping the old one but not the memory associated with it.",N,[[["reusable"],["u"]],["reusable"]]],[11,"fmt","","",0,[[["self"],["formatter"]],["result"]]],[11,"clone","","",0,[[["self"]],["growablepoolbuilder"]]],[11,"eq","","",0,[[["self"],["growablepoolbuilder"]],["bool"]]],[11,"ne","","",0,[[["self"],["growablepoolbuilder"]],["bool"]]],[11,"default","","",0,[[],["self"]]],[11,"new","","Creates a new pool builder with default options.",0,[[],["self"]]],[11,"enable_overgrow","","If set to `false` all returning [`Growable`] will be dropped if there is not enough free space available in a pool.",0,[[["self"],["bool"]],["self"]]],[11,"with_default_capacity","","Sets the default capacity for each allocated [`Growable`].",0,[[["self"],["usize"]],["self"]]],[11,"with_default_ptr_alignment","","Sets the default ptr alignment for each allocated [`Growable`].",0,[[["self"],["usize"]],["self"]]],[11,"with_capacity","","Sets a pool capacity used for every pool reallocation. Note that with `overgrow` enabled it is possible for the pool to grow beyond this capacity. If set to zero the pool will only allocate a [`Growable`] on an explicit allocation request.",0,[[["self"],["usize"]],["self"]]],[11,"build","","Creates a new [`GrowablePool`] using this builder.",0,[[["self"]],["growablepool"]]],[11,"clone","","",1,[[["self"]],["self"]]],[11,"fmt","","",1,[[["self"],["formatter"]],["result"]]],[11,"default","","",1,[[],["self"]]],[11,"new","","Creates a new pool with default options.",1,[[],["self"]]],[11,"builder","","Creates a new pool builder with default options.",1,[[],["growablepoolbuilder"]]],[11,"is_empty","","Returns true if a reallocation will be needed to allocate an another one object.",1,[[["self"]],["bool"]]],[11,"len","","Returns the current amount of allocations that this pool can provide without a reallocation.",1,[[["self"]],["usize"]]],[11,"allocate","","Allocates a new [`Reusable`] from the pool.",1,[[["self"],["t"]],["reusable"]]],[11,"free","","Returns the [`Reusable`] back to the pool, marking it available for a next allocation.",1,[[["self"],["reusable"]]]],[11,"clone","","",2,[[["self"]],["self"]]],[11,"fmt","","",2,[[["self"],["formatter"]],["result"]]],[11,"fmt","","",2,[[["self"],["formatter"]],["result"]]],[11,"default","","",2,[[],["self"]]],[11,"drop","","",2,[[["self"]]]],[11,"new","","Returns a new instance of `Growable` but does not allocate any memory on the heap yet.",2,[[],["self"]]],[11,"with_capacity_for_type","","Returns a new instance of `Growable` with memory already allocated on the heap suitable to store an instance of a given type T.",2,[[],["self"]]],[11,"with_capacity","","Returns a new instance of `Growable` with memory already allocated on the heap.",2,[[["usize"],["usize"]],["self"]]],[11,"is_empty","","Returns true if no memory has been allocated yet.",2,[[["self"]],["bool"]]],[11,"len","","Returns the amount of memory allocated by this `Growable`.",2,[[["self"]],["usize"]]],[11,"alignment","","Returns the alignment.",2,[[["self"]],["usize"]]],[11,"consume","","Places an instance of `T` on the heap, an actual (re)allocation will be performed only if there is not enough space or the pointer alignment is invalid.",2,[[["self"],["t"]],["reusable"]]],[11,"clone","","",3,[[["self"]],["self"]]],[11,"deref","","",3,N],[11,"deref_mut","","",3,N],[11,"fmt","","",3,[[["self"],["formatter"]],["result"]]],[11,"fmt","","",3,[[["self"],["formatter"]],["result"]]],[11,"drop","","",3,[[["self"]]]],[11,"free","","Drops the value and returns the memory back as a [`Growable`].",3,[[["self"]],["growable"]]],[11,"free_move","","Moves the value out of this [`Reusable`] without dropping it and then returns it back with [`Growable`].",3,N],[11,"from","","",0,[[["t"]],["t"]]],[11,"into","","",0,[[["self"]],["u"]]],[11,"to_owned","","",0,[[["self"]],["t"]]],[11,"clone_into","","",0,N],[11,"try_from","","",0,[[["u"]],["result"]]],[11,"borrow","","",0,[[["self"]],["t"]]],[11,"borrow_mut","","",0,[[["self"]],["t"]]],[11,"try_into","","",0,[[["self"]],["result"]]],[11,"get_type_id","","",0,[[["self"]],["typeid"]]],[11,"from","","",1,[[["t"]],["t"]]],[11,"into","","",1,[[["self"]],["u"]]],[11,"to_owned","","",1,[[["self"]],["t"]]],[11,"clone_into","","",1,N],[11,"try_from","","",1,[[["u"]],["result"]]],[11,"borrow","","",1,[[["self"]],["t"]]],[11,"borrow_mut","","",1,[[["self"]],["t"]]],[11,"try_into","","",1,[[["self"]],["result"]]],[11,"get_type_id","","",1,[[["self"]],["typeid"]]],[11,"from","","",2,[[["t"]],["t"]]],[11,"into","","",2,[[["self"]],["u"]]],[11,"to_owned","","",2,[[["self"]],["t"]]],[11,"clone_into","","",2,N],[11,"try_from","","",2,[[["u"]],["result"]]],[11,"borrow","","",2,[[["self"]],["t"]]],[11,"borrow_mut","","",2,[[["self"]],["t"]]],[11,"try_into","","",2,[[["self"]],["result"]]],[11,"get_type_id","","",2,[[["self"]],["typeid"]]],[11,"from","","",3,[[["t"]],["t"]]],[11,"into","","",3,[[["self"]],["u"]]],[11,"to_owned","","",3,[[["self"]],["t"]]],[11,"clone_into","","",3,N],[11,"try_from","","",3,[[["u"]],["result"]]],[11,"borrow","","",3,[[["self"]],["t"]]],[11,"borrow_mut","","",3,[[["self"]],["t"]]],[11,"try_into","","",3,[[["self"]],["result"]]],[11,"get_type_id","","",3,[[["self"]],["typeid"]]]],"paths":[[3,"GrowablePoolBuilder"],[3,"GrowablePool"],[3,"Growable"],[3,"Reusable"]]};
initSearch(searchIndex);
