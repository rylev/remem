use remem::Pool;

#[test]
fn it_works() {
    let pool = Pool::<Vec<u8>>::new(|| Vec::new(), |v| v.clear());
    let mut item = pool.get();
    item.push(1);

    let mut item = pool.get();
    item.push(1);
}
