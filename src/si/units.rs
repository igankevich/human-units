use crate::si::si_unit;

macro_rules! define_si_units {
    { $(($name: ident, $symbol: literal, $description: literal),)* } => {
        $(
            #[doc = $description]
            #[si_unit(symbol = $symbol, internal)]
            pub struct $name(pub u64);
        )*
    };
}

define_si_units! {
    // Base units
    (Time, "s", "Time."),
    (Length, "m", "Length."),
    (Mass, "g", "Mass."),
    (ElectricCurrent, "A", "Electric current."),
    (ThermodynamicTemperature, "K", "Thermodynamic temperature."),
    (AmountOfSubstance, "mol", "Amount of substance."),
    (LuminousIntensity, "cd", "Luminous intensity."),
    // Derived units.
    (PlaneAngle, "rad", "Plane angle."),
    (SolidAngle, "sr", "Solid angle."),
    (Frequency, "Hz", "Frequency."),
    (Force, "N", "Force."),
    (Pressure, "Pa", "Pressure."),
    (Energy, "J", "Energy."),
    (Power, "W", "Power."),
    (ElectricCharge, "C", "Electric charge."),
    (Voltage, "V", "Voltage."),
    (Capacitance, "F", "Capacitance."),
    (ElectricConductance, "S", "Electric conductance."),
    (MagneticFlux, "Wb", "Magnetic flux."),
    (MagneticFluxDensity, "T", "Magnetic flux density."),
    (Inductance, "H", "Inductance."),
    (CelsiusTemperature, "°C", "Celsius temperature."),
    (LuminousFlux, "lm", "Luminous flux."),
    (Illuminance, "lx", "Illuminance."),
    (Radioactivity, "Bq", "Radioactivity."),
    (AbsorbedDose, "Gy", "Absorbed dose."),
    (DoseEquivalent, "Sy", "Dose equivalent."),
    (CatalyticActivity, "kat", "Catalytic activity."),
    // Extra units.
    (Size, "B", "Data size."),
}

#[cfg(feature = "unicode")]
define_si_units! {
    (ElectricResistance, "Ω", "Electric resistance."),
}

#[cfg(not(feature = "unicode"))]
define_si_units! {
    (ElectricResistance, "ohm", "Electric resistance."),
}
