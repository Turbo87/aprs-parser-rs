extern crate aprs_parser;

fn main() {
    let result = aprs_parser::parse(
        r"ICA3D17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054",
    );

    println!("{:#?}", result);
}
