var searchIndex = {};
searchIndex["growable"] = {"doc":"A growable, reusable box for Rust.","items":[[3,"Reusable","growable","Growable with some data assigned to it. It behaves just like default Box does (so it WILL free memory on drop) but also could be freed manually, fetching Growable back.",null,null],[4,"Growable","","A chunk of heap memory that can be assigned to a struct or a trait object. Until assigned to some data it behaves similarly to a Box<[u8; N]>, it can be cloned and would be dropped if leaves the scope.",null,null],[13,"Some","","Pre-allocated chunk of memory.",0,null],[12,"len","growable::Growable","Memory block length.",0,null],[12,"ptr_alignment","","Required alignment for the pointer.",0,null],[12,"ptr","","Pointer.",0,null],[13,"None","growable","No assigned memory.",0,null],[11,"clone","","",0,{"inputs":[{"name":"self"}],"output":{"name":"self"}}],[11,"fmt","","",0,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"drop","","",0,{"inputs":[{"name":"self"}],"output":null}],[11,"new","","Returns a new instance of Growable but does not allocate any memory on the heap yet.",0,{"inputs":[],"output":{"name":"self"}}],[11,"with_capacity","","Returns a new instance of Growable and allocates memory on the heap. In stable Rust it is possible to get a required pointer alignment for any type with align_of function.",0,{"inputs":[{"name":"usize"},{"name":"usize"}],"output":{"name":"self"}}],[11,"len","","Returns the amount of memory allocated by this Growable.",0,{"inputs":[{"name":"self"}],"output":{"name":"usize"}}],[11,"assign","","Returns allocated on the heap struct, an actual (re)allocation will be performed only if there is not enough space in this Growable or the pointer alignment is invalid.",0,{"inputs":[{"name":"self"},{"name":"t"}],"output":{"name":"reusable"}}],[11,"assign_as_trait","","Returns allocated on the heap struct, an actual (re)allocation will be performed only if there is not enough space in this Growable or the pointer alignment is invalid. Additionally stores meta pointer to the vtable creating trait object.",0,{"inputs":[{"name":"self"},{"name":"t"}],"output":{"name":"reusable"}}],[11,"deref","","",1,null],[11,"deref_mut","","",1,null],[11,"drop","","",1,{"inputs":[{"name":"self"}],"output":null}],[11,"free","","Performs drop call on the stored value and returns freed memory block back as a Growable struct.",1,{"inputs":[{"name":"self"}],"output":{"name":"growable"}}]],"paths":[[4,"Growable"],[3,"Reusable"]]};
initSearch(searchIndex);