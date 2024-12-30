#[cfg(test)]
mod columnar_struct_vec_tests {
    use columnar_struct_vec::columnar_struct_vec;

    #[columnar_struct_vec]
    struct Test {
        x: i32,
        y: String,
    }

    #[test]
    fn new() {
        let test = Test::default();
        assert_eq!(test.x.len(), 0);
    }

    #[test]
    fn len() {
        let test = Test::new(vec![0, 1], vec!["a".into(), "b".into()]);
        assert_eq!(test.len(), 2);
    }

    #[columnar_struct_vec]
    struct TestNums {
        pub x: i32,
        y: i32,
    }

    #[test]
    fn push() {
        let t = Test::default();

        let mut builder = t.start_push();
        builder.x(1);
        builder.y("b".into());
        let t = builder.finalize_push().unwrap();

        let mut builder = t.start_push();
        builder.x(2);
        builder.y("c".into());
        let t = builder.finalize_push().unwrap();

        assert_eq!(t.x, vec![1, 2]);
    }

    #[columnar_struct_vec]
    struct TestWithDefault {
        x: f32,
        #[struct_builder(default = 999.0)]
        y: f32,
        #[struct_builder(default = "NaN")]
        z: f32,
    }

    #[test]
    fn partial_push() {
        let t = TestWithDefault::default();
        let mut builder = t.start_push();
        builder.x(1.0);
        let t = builder.finalize_push().unwrap();
        assert_eq!(t.y[0], 999.0);
        assert!(t.z[0].is_nan());
    }

    #[test]
    fn push_missing_default() {
        let t = TestWithDefault::default();
        let mut builder = t.start_push();
        builder.y(1.0);
        let result = builder.finalize_push();
        assert!(result.is_err());
    }

    #[columnar_struct_vec]
    struct TestWithDate {
        x: f32,
        date: i64,
    }

    #[columnar_struct_vec]
    struct TestWithUSize {
        x: f32,
        y: usize,
    }

    #[repr(u8)]
    #[derive(Debug, PartialEq, Copy, Clone)]
    enum TestEnum {
        A = 0,
        B = 1,
    }

    impl std::fmt::Display for TestEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            return write!(f, "{:?}", self);
        }
    }

    #[columnar_struct_vec]
    struct TestWithEnum {
        x: f32,
        y: TestEnum,
    }

    #[test]
    fn with_enum() {
        let test = TestWithEnum::new(vec![1., 2.], vec![TestEnum::A, TestEnum::B]);
        assert_eq!(test.y, vec![TestEnum::A, TestEnum::B]);
    }

    #[columnar_struct_vec]
    struct TestWithStaticStr {
        pub x: f32,
        y: &'static str,
    }

    #[test]
    fn with_static_str() {
        let test = TestWithStaticStr::new(vec![1., 2.], vec!["a", "b"]);
        assert_eq!(test.y, vec!["a", "b"]);
    }

    #[columnar_struct_vec]
    struct TestWithString {
        pub x: i32,
        y: String,
    }

    #[test]
    fn with_string() {
        let test = TestWithString::new(vec![1, 2], vec!["a".to_string(), "b".to_string()]);
        assert_eq!(test.y, vec!["a", "b"]);
    }

    #[columnar_struct_vec]
    struct TestWithStringWithDefault {
        pub x: i32,
        #[struct_builder(default)]
        y: String,
    }

    #[test]
    fn with_string_with_default() {
        let test = TestWithStringWithDefault::default();
        let mut builder = test.start_push();
        builder.x(0);
        let test = builder.finalize_push().unwrap();
        assert_eq!(test.y, vec![""]);
    }
}
