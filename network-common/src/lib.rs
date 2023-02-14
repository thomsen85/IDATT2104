pub mod thread_pool;
pub mod http;
pub mod bitbuilder;
pub mod websockets;

#[cfg(test)]
mod tests {
    use crate::bitbuilder::BitBuilder;

    #[test]
    fn test_bitbuilder_get_bit_gets_bit_1() {
        let mut builder = BitBuilder::new();

        builder.push_bit(true);
        builder.push_bit(false);
        builder.push_bit(true);

        dbg!(&builder);

        assert_eq!(builder.get_bit(0), Some(true));
        assert_eq!(builder.get_bit(1), Some(false));
        assert_eq!(builder.get_bit(2), Some(true));
    }

    #[test]
    fn test_bitbuilder_get_bit_gets_bit_2() {
        let mut builder = BitBuilder::new();

        builder.push_byte(u8::MAX);

        builder.push_bit(true);
        builder.push_bit(false);
        builder.push_bit(false);

        dbg!(&builder);

        assert_eq!(builder.get_bit(0+8), Some(true));
        assert_eq!(builder.get_bit(1+8), Some(false));
        assert_eq!(builder.get_bit(2+8), Some(false));
    }

    #[test]
    fn test_bitbuidler_get_bit_gets_none_when_out_of_index() {
        let mut builder = BitBuilder::new();

        builder.push_bit(true);
        builder.push_bit(false);

        dbg!(&builder);

        assert_eq!(builder.get_bit(2), None);
        assert_eq!(builder.get_bit(20), None);

    }

    #[test]
    fn test_bitbuilder_from_bit_string() {
        let builder = BitBuilder::from_bit_string("001010101");

        dbg!(&builder);

        assert_eq!(builder.get_bit(0), Some(false));
        assert_eq!(builder.get_bit(1), Some(false));
        assert_eq!(builder.get_bit(2), Some(true));
        assert_eq!(builder.get_bit(3), Some(false));
        assert_eq!(builder.get_bit(4), Some(true));
        assert_eq!(builder.get_bit(5), Some(false));
        assert_eq!(builder.get_bit(6), Some(true));
        assert_eq!(builder.get_bit(7), Some(false));
        assert_eq!(builder.get_bit(8), Some(true));

    }
}