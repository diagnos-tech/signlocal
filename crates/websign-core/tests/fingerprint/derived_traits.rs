//! Derived traits.

use super::*;

#[test]
fn is_copy_and_compares_by_bytes() {
    let a = Fingerprint::from_bytes([1; 32]);
    let b = a;
    assert_eq!(a, b);
    assert_ne!(a, Fingerprint::from_bytes([2; 32]));
    let mut last_differs = [1; 32];
    last_differs[31] = 2;
    assert_ne!(a, Fingerprint::from_bytes(last_differs));
}

#[test]
fn hashes_consistently_with_equality() {
    let set: HashSet<Fingerprint> = [[1u8; 32], [2; 32], [1; 32]]
        .into_iter()
        .map(Fingerprint::from_bytes)
        .collect();
    assert_eq!(set.len(), 2);
    assert!(set.contains(&Fingerprint::from_bytes([2; 32])));
}

#[test]
fn orders_lexicographically_by_bytes() {
    let mut low = [0u8; 32];
    low[31] = 1;
    let mut mid = [0u8; 32];
    mid[0] = 1;
    let high = [0xFF; 32];
    let sorted = [
        Fingerprint::from_bytes([0; 32]),
        Fingerprint::from_bytes(low),
        Fingerprint::from_bytes(mid),
        Fingerprint::from_bytes(high),
    ];
    assert!(sorted.windows(2).all(|w| w[0] < w[1]));
    let shuffled: BTreeSet<Fingerprint> = [sorted[3], sorted[1], sorted[0], sorted[2]]
        .into_iter()
        .collect();
    assert_eq!(shuffled.into_iter().collect::<Vec<_>>(), sorted);
    assert_eq!(sorted[1].cmp(&sorted[1]), std::cmp::Ordering::Equal);
}
