pub fn convert_unit_to_si(mut num:f64, unit:&str) -> Option<f64> {
    match unit {
        "pi" => {num *= 3.141592653589793},
        "sqrt2" => {num *= 1.414213562373095},
        "sqrt0.5" => {num *= 0.7071067811865476},
        "sqrt2pi" => {num *= 2.50662827463100050241576528481104525},
        // this is kind of confusing with 10e 5 looks like 10 exponent 5
        //"e" => {num *= 2.718281828459045},
        "ln2" => {num *= 0.69314718056},
        "ln10" => {num *= 2.30258509299},
        "log10e" => {num *= 0.4342944819032518},
        "log2e" => {num *= 1.4426950408889634},
        "phi" => {num *= 1.618033988749894},
        "GT" => {num *= 1000000000000.0},
        "MT" => {num *= 1000000000.0},
        "kT" => {num *= 1000000.0},
        "T" => {num *= 1000.0},
        "kg" => {},
        "g" => {num *= 0.001},
        "mg" => {num *= 0.000001},
        "ug" => {num *= 0.000000001},
        "ng" => {num *= 0.000000000001},
        "Ti" => {num *= 1024.0 * 1024.0 * 1024.0 * 1024.0},
        "Gi" => {num *= 1024.0 * 1024.0 * 1024.0},
        "Mi" => {num *= 1024.0 * 1024.0},
        "ki" => {num *= 1024.0},
        "d" => {num *= 60.0 * 60.0 * 24.0},
        "h" => {num *= 60.0 * 60.0},
        "min" => {num *= 60.0},
        "s" => {},
        "ms" => {num *= 0.001},
        "us" => {num *= 0.000001},
        "ns" => {num *= 0.000000001},
        "deg" => {num *= std::f64::consts::PI / 180.0},
        "rad" => {},
        "in" => {num *= 0.024},
        "km" => {num *= 1000.0},
        "m" => {},
        "dm" => {num *= 0.1},
        "cm" => {num *= 0.01},
        "mm" => {num *= 0.001},
        "um" => {num *= 0.000001},
        "nm" => {num *= 0.000000001},
        "lb" => {num *= 0.453592},
        "oz" => {num *= 0.0283495},
        "mile" => {num *= 1609.34},
        "miles" => {num *= 1609.34},
        "ft" => {num *= 0.3048},
        "yd" => {num *= 0.9144},
        "F" => {num = ((num - 32.0) * 5.0 / 9.0) + 273.15},
        "C" => {num = num + 273.15},
        "K" => {},
        "m3" => {},
        "l" => {num *= 0.0001},
        "dm3" => {num *= 0.0001},
        "dl" => {num *= 0.00001},
        "cl" => {num *= 0.000001},
        "ml" => {num *= 0.0000001},
        "cm3" => {num *= 0.0000001},
        "barrel" => {num *= 0.158987294928},
        "cu.ft" => {num *= 0.028},
        "ft3" => {num *= 0.028},
        "gal" => {num *= 0.003785411784},
        "pint" => {num *= 0.000473176473},
        "cu.in" => {num *= 0.000016387064},
        "in3" => {num *= 0.000016387064},
        "cu.yd" => {num *= 0.7645549},
        "yd3" => {num *= 0.7645549},
        "m2" => {},
        "dm2" => {num *= 0.01},
        "cm2" => {num *= 0.0001},
        "mm2" => {num *= 0.000001},
        "a" => {num *= 100.0},
        "ha" => {num *= 100000.0},
        "km2" => {num *= 1000000.0},
        "mile2" => {num *= 2589975.23456},
        "yd2" => {num *= 0.836127},
        "sq.yd" => {num *= 0.836127},
        "ft2" => {num *= 0.092903},
        "sq.ft" => {num *= 0.092903},
        "in2" => {num *= 0.00064516},
        "sq.in" => {num *= 0.00064516},
        _ => {
            return None;
        }
    }
    return Some(num);
}
