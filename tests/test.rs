#![feature(collections, collections_bound, range_inclusive)]

extern crate btree_rewrite;
extern crate collections;
extern crate rand;

use collections::Bound::{self, Included, Excluded, Unbounded};
use btree_rewrite::map::*;
use std::iter::range_inclusive;
use std::rc::Rc;

#[test]
fn test_send_sync() {
    fn t<T: Send + Sync>() {}

    t::<BTreeMap<u32, i32>>();

    t::<IntoIter<u32, i32>>();
    t::<Iter<u32, i32>>();
    t::<IterMut<u32, i32>>();

    t::<Range<u32, i32>>();
    t::<RangeMut<u32, i32>>();

    t::<Entry<u32, i32>>();
    t::<OccupiedEntry<u32, i32>>();
    t::<VacantEntry<u32, i32>>();
}

#[test]
pub fn test_rand_large() {
    use rand::{thread_rng, Rng};

    let n: usize = 50;
    let mut map = BTreeMap::new();
    // setup
    let mut rng = thread_rng();
    let mut log = Vec::new();

    for _ in 0..n {
        let i = rng.gen::<usize>() % (2*n);
        if !map.contains_key(&i) {
            log.push(i);
        }
        map.insert(i, i);
//        map.dump();
    }


    for i in log {
        assert_eq!(map.get(&i).cloned(), Some(i));
    }
}

#[test]
fn test_basic_large() {
    let mut map = BTreeMap::new();
    let size = 10000;
    assert_eq!(map.len(), 0);

    for i in 0..size {
        assert_eq!(map.insert(i, 10*i), None);
        assert_eq!(map.len(), i + 1);
    }

    for i in 0..size {
        assert_eq!(map.get(&i).unwrap(), &(i*10));
    }

    for i in size..size*2 {
        assert_eq!(map.get(&i), None);
    }

    for i in 0..size {
        assert_eq!(map.insert(i, 100*i), Some(10*i));
        assert_eq!(map.len(), size);
    }

    for i in 0..size {
        assert_eq!(map.get(&i).unwrap(), &(i*100));
    }

    for i in 0..size/2 {
        assert_eq!(map.remove(&(i*2)), Some(i*200));
        assert_eq!(map.len(), size - i - 1);
    }

    for i in 0..size/2 {
        assert_eq!(map.get(&(2*i)), None);
        assert_eq!(map.get(&(2*i+1)).unwrap(), &(i*200 + 100));
    }

    for i in 0..size/2 {
        assert_eq!(map.remove(&(2*i)), None);
        assert_eq!(map.remove(&(2*i+1)), Some(i*200 + 100));
        assert_eq!(map.len(), size/2 - i - 1);
    }
}

#[test]
fn test_basic_small() {
    let mut map = BTreeMap::new();
    assert_eq!(map.remove(&1), None);
    assert_eq!(map.get(&1), None);
    assert_eq!(map.insert(1, 1), None);
    assert_eq!(map.get(&1), Some(&1));
    assert_eq!(map.insert(1, 2), Some(1));
    assert_eq!(map.get(&1), Some(&2));
    assert_eq!(map.insert(2, 4), None);
    assert_eq!(map.get(&2), Some(&4));
    assert_eq!(map.remove(&1), Some(2));
    assert_eq!(map.remove(&2), Some(4));
    assert_eq!(map.remove(&1), None);
}

#[test]
fn test_iter() {
    let size = 10000;

    // Forwards
    let mut map: BTreeMap<_, _> = (0..size).map(|i| (i, i)).collect();

    fn test<T>(size: usize, mut iter: T) where T: Iterator<Item=(usize, usize)> {
        for i in 0..size {
            assert_eq!(iter.size_hint(), (size - i, Some(size - i)));
            assert_eq!(iter.next().unwrap(), (i, i));
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }
    test(size, map.iter().map(|(&k, &v)| (k, v)));
    test(size, map.iter_mut().map(|(&k, &mut v)| (k, v)));
    test(size, map.into_iter());
}


#[test]
fn test_iter_rev() {
    let size = 10000;

    // Forwards
    let mut map: BTreeMap<_, _> = (0..size).map(|i| (i, i)).collect();

    fn test<T>(size: usize, mut iter: T) where T: Iterator<Item=(usize, usize)> {
        for i in 0..size {
            assert_eq!(iter.size_hint(), (size - i, Some(size - i)));
            assert_eq!(iter.next().unwrap(), (size - i - 1, size - i - 1));
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }
    test(size, map.iter().rev().map(|(&k, &v)| (k, v)));
    test(size, map.iter_mut().rev().map(|(&k, &mut v)| (k, v)));
    test(size, map.into_iter().rev());
}

#[test]
fn test_iter_mixed() {
    let size = 10000;

    // Forwards
    let mut map: BTreeMap<_, _> = (0..size).map(|i| (i, i)).collect();

    fn test<T>(size: usize, mut iter: T)
            where T: Iterator<Item=(usize, usize)> + DoubleEndedIterator {
        for i in 0..size / 4 {
            assert_eq!(iter.size_hint(), (size - i * 2, Some(size - i * 2)));
            assert_eq!(iter.next().unwrap(), (i, i));
            assert_eq!(iter.next_back().unwrap(), (size - i - 1, size - i - 1));
        }
        for i in size / 4..size * 3 / 4 {
            assert_eq!(iter.size_hint(), (size * 3 / 4 - i, Some(size * 3 / 4 - i)));
            assert_eq!(iter.next().unwrap(), (i, i));
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }
    test(size, map.iter().map(|(&k, &v)| (k, v)));
    test(size, map.iter_mut().map(|(&k, &mut v)| (k, v)));
    test(size, map.into_iter());
}

#[test]
fn test_range_small() {
    let size = 5;

    // Forwards
    let map: BTreeMap<_, _> = (0..size).map(|i| (i, i)).collect();

    let mut j = 0;
    for ((&k, &v), i) in map.range(Included(&2), Unbounded).zip(2..size) {
        assert_eq!(k, i);
        assert_eq!(v, i);
        j += 1;
    }
    assert_eq!(j, size - 2);
}

#[test]
fn test_range_1000() {
    let size = 1000;
    let map: BTreeMap<_, _> = (0..size).map(|i| (i, i)).collect();

    fn test(map: &BTreeMap<u32, u32>, size: u32, min: Bound<&u32>, max: Bound<&u32>) {
        let mut kvs = map.range(min, max).map(|(&k, &v)| (k, v));
        let mut pairs = (0..size).map(|i| (i, i));

        for (kv, pair) in kvs.by_ref().zip(pairs.by_ref()) {
            assert_eq!(kv, pair);
        }
        assert_eq!(kvs.next(), None);
        assert_eq!(pairs.next(), None);
    }
    test(&map, size, Included(&0), Excluded(&size));
    test(&map, size, Unbounded, Excluded(&size));
    test(&map, size, Included(&0), Included(&(size - 1)));
    test(&map, size, Unbounded, Included(&(size - 1)));
    test(&map, size, Included(&0), Unbounded);
    test(&map, size, Unbounded, Unbounded);
}

#[test]
fn test_range() {
    let size = 200;
    let map: BTreeMap<_, _> = (0..size).map(|i| (i, i)).collect();

    for i in 0..size {
        for j in i..size {
            let mut kvs = map.range(Included(&i), Included(&j)).map(|(&k, &v)| (k, v));
            let mut pairs = range_inclusive(i, j).map(|i| (i, i));

            for (kv, pair) in kvs.by_ref().zip(pairs.by_ref()) {
                assert_eq!(kv, pair);
            }
            assert_eq!(kvs.next(), None);
            assert_eq!(pairs.next(), None);
        }
    }
}

#[test]
fn test_borrow() {
    // make sure these compile -- using the Borrow trait
    {
        let mut map = BTreeMap::new();
        map.insert("0".to_string(), 1);
        assert_eq!(map["0"], 1);
    }

    {
        let mut map = BTreeMap::new();
        map.insert(Box::new(0), 1);
        assert_eq!(map[&0], 1);
    }

    {
        let mut map = BTreeMap::new();
        map.insert(Box::new([0, 1]) as Box<[i32]>, 1);
        assert_eq!(map[&[0, 1][..]], 1);
    }

    {
        let mut map = BTreeMap::new();
        map.insert(Rc::new(0), 1);
        assert_eq!(map[&0], 1);
    }
}

#[test]
fn test_entry(){
    let xs = [(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)];

    let mut map: BTreeMap<_, _> = xs.iter().cloned().collect();

    // Existing key (insert)
    match map.entry(1) {
        Vacant(_) => unreachable!(),
        Occupied(mut view) => {
            assert_eq!(view.get(), &10);
            assert_eq!(view.insert(100), 10);
        }
    }
    assert_eq!(map.get(&1).unwrap(), &100);
    assert_eq!(map.len(), 6);


    // Existing key (update)
    match map.entry(2) {
        Vacant(_) => unreachable!(),
        Occupied(mut view) => {
            let v = view.get_mut();
            *v *= 10;
        }
    }
    assert_eq!(map.get(&2).unwrap(), &200);
    assert_eq!(map.len(), 6);

    // Existing key (take)
    match map.entry(3) {
        Vacant(_) => unreachable!(),
        Occupied(view) => {
            assert_eq!(view.remove(), 30);
        }
    }
    assert_eq!(map.get(&3), None);
    assert_eq!(map.len(), 5);


    // Inexistent key (insert)
    match map.entry(10) {
        Occupied(_) => unreachable!(),
        Vacant(view) => {
            assert_eq!(*view.insert(1000), 1000);
        }
    }
    assert_eq!(map.get(&10).unwrap(), &1000);
    assert_eq!(map.len(), 6);
}

#[test]
fn test_extend_ref() {
    let mut a = BTreeMap::new();
    a.insert(1, "one");
    let mut b = BTreeMap::new();
    b.insert(2, "two");
    b.insert(3, "three");

    a.extend(&b);

    assert_eq!(a.len(), 3);
    assert_eq!(a[&1], "one");
    assert_eq!(a[&2], "two");
    assert_eq!(a[&3], "three");
}
