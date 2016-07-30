use SecBox;

#[test]
fn test_new() {
    let a = SecBox::new(b'a');
    let b = SecBox::new(b'b');
    let c = SecBox::new(b'c');

    assert_eq!(*a, b'a');
    assert_eq!(*b, b'b');
    assert_eq!(*c, b'c');
}

#[test]
fn test_unsized() {
    let string = String::from("abcs").into_boxed_str();

    let bx = SecBox::from(string);

    assert_eq!(&*bx, "abcs");
}

#[test]
fn test_zeroed() {
    let bx = SecBox::new(44);

    let ptr = &*bx as *const i32;

    drop(bx);

    unsafe {
        assert_eq!(*ptr, 0);
    }
}

#[test]
fn test_into_inner() {
    let a = SecBox::new(b'a');
    let b = SecBox::new(b'b');
    let c = SecBox::new(b'c');

    assert_eq!(a.into_inner(), b'a');
    assert_eq!(b.into_inner(), b'b');
    assert_eq!(c.into_inner(), b'c');
}

#[test]
fn test_mut() {
    let mut n = SecBox::new(0);

    assert_eq!(*n, 0);

    *n += 1;

    assert_eq!(*n, 1);

    *n = 55;

    assert_eq!(*n, 55);
}

#[test]
fn test_clone() {
    let bx = SecBox::new(0);
    let mut bx2 = bx.clone();

    *bx2 = 3;

    assert_eq!(*bx, 0);
    assert_eq!(*bx2, 3);
}

#[test]
fn test_clone_from() {
    let bx = SecBox::new(0);
    let mut bx2 = SecBox::new(44);

    bx2.clone_from(&bx);

    assert_eq!(*bx, 0);
    assert_eq!(*bx2, 0);
}
