# remem

Utility for reusing pieces of memory

## Usage

``` rust
let pool = Pool::<Vec<u8>>::new(|| Vec::new());
let mut item = pool.get();

item.push(1);
item.push(2);
item.push(3);

drop(item);

// item's memory can now be reused

let mut item = pool.get();

item.push(1);
item.push(2);
item.push(3);
```
