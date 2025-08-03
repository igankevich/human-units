use human_units::iec;
use human_units::si;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    generate_si_tests();
    generate_iec_tests();
}

fn generate_si_tests() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&out_dir).join("si_tests.rs")).unwrap();
    writeln!(&mut file, "parameterize! {{").unwrap();
    for (uint, max) in [
        ("u128", u128::MAX),
        ("u64", u64::MAX as u128),
        ("u32", u32::MAX as u128),
        ("u16", u16::MAX as u128),
    ] {
        let ilog1000 = max.ilog(1000);
        for (i, a) in si::Prefix::ALL.iter().copied().enumerate() {
            for b in si::Prefix::ALL.iter().copied().skip(i) {
                if b as u32 - a as u32 > ilog1000 {
                    continue;
                }
                let module = format!("{uint}_{a:?}_{b:?}").to_lowercase();
                writeln!(
                    &mut file,
                    "({} {uint} {:?} {:?} {:?} {:?})",
                    module,
                    a.as_str(),
                    b.as_str(),
                    a,
                    b
                )
                .unwrap();
            }
        }
    }
    writeln!(&mut file, "}}").unwrap();
}

fn generate_iec_tests() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&out_dir).join("iec_tests.rs")).unwrap();
    writeln!(&mut file, "parameterize! {{").unwrap();
    for (uint, max) in [
        ("u128", u128::MAX),
        ("u64", u64::MAX as u128),
        ("u32", u32::MAX as u128),
        ("u16", u16::MAX as u128),
    ] {
        let ilog1024 = max.ilog(1024);
        for (i, a) in iec::Prefix::ALL.iter().copied().enumerate() {
            for b in iec::Prefix::ALL.iter().copied().skip(i) {
                if b as u32 - a as u32 > ilog1024 {
                    continue;
                }
                let module = format!("{uint}_{a:?}_{b:?}").to_lowercase();
                writeln!(
                    &mut file,
                    "({} {uint} {:?} {:?} {:?} {:?})",
                    module,
                    a.as_str(),
                    b.as_str(),
                    a,
                    b
                )
                .unwrap();
            }
        }
    }
    writeln!(&mut file, "}}").unwrap();
}
