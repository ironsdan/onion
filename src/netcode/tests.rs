
#[cfg(test)]
mod tests {
    use crate::netcode::replay;
    #[test]
    fn test_simple_current() {
        let mut r = replay::Replayable::new(|input: &i8, state: &i8| -> i8 {
            input + state
        }, 0, 0);
        assert_eq!(0, *r.current())
    }

    #[test]
    fn test_add_one() {
        let mut r = replay::Replayable::new(|input: &i8, state: &i8| -> i8 {
            input + state
        }, 0, 0);
        r.advance(1);
        assert_eq!(1, *r.current())
    }

    #[test]
    fn test_prehistory() {
        let mut r = replay::Replayable::new(|input: &i8, state: &i8| -> i8 {
            input + state
        }, 0, 0);
        r.force(10, 0, 0);
        r.update_input(9, |i: &mut i8| {
            *i = 10;
        });
        assert_eq!(0, *r.current())
    }

    #[test]
    fn test_update_history() {
        let mut r = replay::Replayable::new(|input: &i64, state: &i64| -> i64 {
            input * state
        }, 1, 1);
        r.advance(2);
        r.advance(2);
        r.advance(2);
        assert_eq!(8, *r.current());

        r.update_input(2, |i: &mut i64| {*i = 0});
        assert_eq!(0, *r.current());

        r.update_input(2, |i: &mut i64| {*i = 3});
        assert_eq!(12, *r.current());

        r.update_input(8, |i: &mut i64| {*i = 2});
        assert_eq!(384, *r.current());
    }

}
