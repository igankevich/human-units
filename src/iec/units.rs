use crate::iec::iec_unit;

macro_rules! define_units {
    { $((
        $name: ident
        $symbol: literal
        $description: literal
    ))* } => {
        $(
            #[doc = $description]
            #[iec_unit(symbol = $symbol, internal)]
            pub struct $name(pub u64);
        )*
    };
}

define_units! {
    // Information.
    (Bit "bit" "The basic unit of information.")
    (Byte "B" "The unit of information.")
    (Octet "o" "The unit of information equal to 8 bits.")
    (Shannon "Sh" "The unit of information.")
    (Nat "Nat" "The natural unit of information.")
    (Hartley "Hart" "The logarithmic unit of information.")
    (Erlang "E" "Traffic intensity in percentage per hour.")
    (Baud "Bd" "Modulation rate in symbols per second.")
}
