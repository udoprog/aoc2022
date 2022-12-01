use super::Buf;

#[test]
fn test_read_write_unaligned() {
    let mut b = Buf::<11>::new();

    let data = [1, 2, 3, 4, 5];

    for _ in 0..128 {
        assert_eq!(b.write(&data[..3]), 3);
        assert_eq!(b.write(&data[3..]), 2);
        let mut buf = [0; 8];

        assert_eq!(b.read(&mut buf[..3]), 3);
        assert_eq!(&buf[..3], &[1, 2, 3]);

        assert_eq!(b.read(&mut buf[..]), 2);
        assert_eq!(&buf[..2], &[4, 5]);
    }
}
