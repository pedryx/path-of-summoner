pub fn num_to_roman(num: u8) -> &'static str {
    match num {
        0 => "0",
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        6 => "VI",
        7 => "VII",
        8 => "VIII",
        9 => "IX",
        10 => "X",
        _ => panic!("Number {} not supported!", num),
    }
}
