use crate::si::define_si_unit;

// Base units.
define_si_unit!(Time, u64, "s", "Time.", time);
define_si_unit!(Length, u64, "m", "Length.", length);
define_si_unit!(Mass, u64, "g", "Mass.", mass);
define_si_unit!(
    ElectricCurrent,
    u64,
    "A",
    "Electric current.",
    electric_current
);
define_si_unit!(
    ThermodynamicTemperature,
    u64,
    "K",
    "Thermodynamic temperature.",
    thermodynamic_temperature
);
define_si_unit!(
    AmountOfSubstance,
    u64,
    "mol",
    "Amount of substance.",
    amount_of_substance
);
define_si_unit!(
    LuminousIntensity,
    u64,
    "cd",
    "Luminous intensity.",
    luminous_intensity
);

// Derived units.
define_si_unit!(PlaneAngle, u64, "rad", "Plane angle.", plane_angle);
define_si_unit!(SolidAngle, u64, "sr", "Solid angle.", solid_angle);
define_si_unit!(Frequency, u64, "Hz", "Frequency.", frequency);
define_si_unit!(Force, u64, "N", "Force.", force);
define_si_unit!(Pressure, u64, "Pa", "Pressure.", pressure);
define_si_unit!(Energy, u64, "J", "Energy.", energy);
define_si_unit!(Power, u64, "W", "Power.", power);
define_si_unit!(
    ElectricCharge,
    u64,
    "C",
    "Electric charge.",
    electric_charge
);
define_si_unit!(Voltage, u64, "V", "Voltage.", voltage);
define_si_unit!(Capacitance, u64, "F", "Capacitance.", capacitance);
define_si_unit!(
    ElectricResistance,
    u64,
    crate::si::unicode!("Ω", "ohm"),
    "Electric resistance.",
    electric_resistance
);
define_si_unit!(
    ElectricConductance,
    u64,
    "S",
    "Electric conductance.",
    electric_conductance
);
define_si_unit!(MagneticFlux, u64, "Wb", "Magnetic flux.", magnetic_flux);
define_si_unit!(
    MagneticFluxDensity,
    u64,
    "T",
    "Magnetic flux density.",
    magnetic_flux_density
);
define_si_unit!(Inductance, u64, "H", "Inductance.", inductance);
define_si_unit!(
    CelsiusTemperature,
    u64,
    "°C",
    "Celsius temperature.",
    celsius_temperature
);
define_si_unit!(LuminousFlux, u64, "lm", "Luminous flux.", luminous_flux);
define_si_unit!(Illuminance, u64, "lx", "Illuminance.", illuminance);
define_si_unit!(Radioactivity, u64, "Bq", "Radioactivity.", radioactivity);
define_si_unit!(AbsorbedDose, u64, "Gy", "Absorbed dose.", absorbed_dose);
define_si_unit!(
    DoseEquivalent,
    u64,
    "Sy",
    "Dose equivalent.",
    dose_equivalent
);
define_si_unit!(
    CatalyticActivity,
    u64,
    "kat",
    "Catalytic activity.",
    catalytic_activity
);
