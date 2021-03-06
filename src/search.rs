use collections::borrow::Borrow;
use core::cmp::Ordering;

use node::{Handle, NodeRef, marker};

use node::ForceResult::*;
use self::SearchResult::*;

pub enum SearchResult<Lifetime, K, V, Mutability, FoundType, GoDownType> {
    Found(Handle<NodeRef<Lifetime, K, V, Mutability, FoundType>, marker::KV>),
    GoDown(Handle<NodeRef<Lifetime, K, V, Mutability, GoDownType>, marker::Edge>)
}

pub fn search_tree<Lifetime, K, V, Mutability, Q: ?Sized>(mut node: NodeRef<Lifetime, K, V, Mutability, marker::LeafOrInternal>, key: &Q) -> SearchResult<Lifetime, K, V, Mutability, marker::LeafOrInternal, marker::Leaf> where Q: Ord, K: Borrow<Q> {
    loop {
        match search_node(node, key) {
            Found(handle) => return Found(handle),
            GoDown(handle) => match handle.force() {
                Leaf(leaf) => return GoDown(leaf),
                Internal(internal) => {
                    node = internal.descend();
                    continue;
                }
            }
        }
    }
}

pub fn search_node<Lifetime, K, V, Mutability, Type, Q: ?Sized>(node: NodeRef<Lifetime, K, V, Mutability, Type>, key: &Q) -> SearchResult<Lifetime, K, V, Mutability, Type, Type> where Q: Ord, K: Borrow<Q> {
    match search_linear(&node, key) {
        (idx, true) => Found(
            unsafe { Handle::new(node, idx) }
        ),
        (idx, false) => SearchResult::GoDown(
            unsafe { Handle::new(node, idx) }
        )
    }
}

fn search_linear<Lifetime, K, V, Mutability, Type, Q: ?Sized>(node: &NodeRef<Lifetime, K, V, Mutability, Type>, key: &Q) -> (usize, bool) where Q: Ord, K: Borrow<Q> {
    for (i, k) in node.keys().iter().enumerate() {
        match key.cmp(k.borrow()) {
            Ordering::Greater => {},
            Ordering::Equal => return (i, true),
            Ordering::Less => return (i, false)
        }
    }
    (node.keys().len(), false)
}

