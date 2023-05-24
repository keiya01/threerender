pub fn count_some<T>(v: &Vec<Option<T>>) -> usize {
    let mut cnt = 0;
    for v in v {
        if v.is_some() {
            cnt += 1;
        }
    }
    cnt
}

#[test]
fn test_count_some() {
    assert_eq!(count_some(&vec![Some(1), None, Some(1), Some(1), None,]), 3);
}
