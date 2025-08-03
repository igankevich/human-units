#![allow(deprecated)]

use crate::si::si_unit;

macro_rules! define_units {
    { $((
        $name: ident
        $symbol: literal
        $description: literal
    ))* } => {
        $(
            #[doc = $description]
            #[si_unit(symbol = $symbol, internal)]
            pub struct $name(pub u64);
        )*
    };
}

define_units! {
    // Base units
    (Time "s" "Time.")
    (Length "m" "Length.")
    (Mass "g" "Mass.")
    (ElectricCurrent "A" "Electric current.")
    (ThermodynamicTemperature "K" "Thermodynamic temperature.")
    (AmountOfSubstance "mol" "Amount of substance.")
    (LuminousIntensity "cd" "Luminous intensity.")
    // Derived units.
    (PlaneAngle "rad" "Plane angle.")
    (SolidAngle "sr" "Solid angle.")
    (Frequency "Hz" "Frequency.")
    (Force "N" "Force.")
    (Pressure "Pa" "Pressure.")
    (Energy "J" "Energy.")
    (Power "W" "Power.")
    (ElectricCharge "C" "Electric charge.")
    (Voltage "V" "Voltage.")
    (Capacitance "F" "Capacitance.")
    (ElectricConductance "S" "Electric conductance.")
    (MagneticFlux "Wb" "Magnetic flux.")
    (MagneticFluxDensity "T" "Magnetic flux density.")
    (Inductance "H" "Inductance.")
    (CelsiusTemperature "°C" "Celsius temperature.")
    (LuminousFlux "lm" "Luminous flux.")
    (Illuminance "lx" "Illuminance.")
    (Radioactivity "Bq" "Radioactivity.")
    (AbsorbedDose "Gy" "Absorbed dose.")
    (DoseEquivalent "Sy" "Dose equivalent.")
    (CatalyticActivity "kat" "Catalytic activity.")
}

#[cfg(feature = "unicode")]
define_units! {
    (ElectricResistance "Ω" "Electric resistance.")
}

#[cfg(not(feature = "unicode"))]
define_units! {
    (ElectricResistance "ohm" "Electric resistance.")
}

/// Data size.
#[si_unit(symbol = "B", min_prefix = "", max_prefix = "exa", internal)]
pub struct Size(pub u64);

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    extern crate alloc;

    #[test]
    fn from_si_works() {
        let cpu_freq = Frequency::from_si(2_200_000_000);
        assert_eq!("2200 MHz", cpu_freq.to_string());
        assert_eq!("2.2 GHz", cpu_freq.format_si().to_string());
    }

    #[test]
    fn size_works() {
        let size = Size::from_si(1_536_000);
        assert_eq!("1536 kB", size.to_string());
        assert_eq!("1.5 MB", size.format_si().to_string());
    }
}
