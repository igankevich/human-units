use paste::paste;

macro_rules! parameterize {
    ($($uint: ident)+) => {
        paste! {
            $(
                pub const fn [<$uint _is_multiple_of>](a: $uint, b: $uint) -> bool {
                    match b {
                        0 => a == 0,
                        _ => a % b == 0,
                    }
                }
            )+
        }
    };
}

parameterize! {
    u128
    u64
    u32
    u16
}
