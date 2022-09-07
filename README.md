
aprs-parser
==============================================================================

[![Build Status](https://travis-ci.org/Turbo87/aprs-parser-rs.svg?branch=master)](https://travis-ci.org/Turbo87/aprs-parser-rs)

[APRS] message parser for [Rust]

[APRS]: http://www.aprs.org/
[Rust]: https://www.rust-lang.org/


Usage
------------------------------------------------------------------------------

```rust
extern crate aprs_parser;

fn main() {
    let result = aprs_parser::parse(
        r"ICA3D17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054"
    );

    println!("{:#?}", result);

    // Ok(
    //     AprsPacket {
    //         from: Callsign {
    //             call: "ICA3D17F2",
    //             ssid: None
    //         },
    //         to: Callsign {
    //             call: "APRS",
    //             ssid: None
    //         },
    //         via: [
    //             Callsign {
    //                 call: "qAS",
    //                 ssid: None
    //             },
    //             Callsign {
    //                 call: "dl4mea",
    //                 ssid: None
    //             }
    //         ],
    //         data: Position(
    //             AprsPosition {
    //                 timestamp: Some(
    //                     HHMMSS(
    //                         7,
    //                         48,
    //                         49
    //                     )
    //                 ),
    //                 messaging_supported: false
    //                 latitude: Latitude(48.360165),
    //                 longitude: Longitude(12.408166),
    //                 symbol_table: '\\',
    //                 symbol_code: '^',
    //                 comment: "322/103/A=003054"
    //             }
    //         )
    //     }
    // )
}
```


License
------------------------------------------------------------------------------

This project is licensed under either of

 - Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   <http://www.apache.org/licenses/LICENSE-2.0>)
   
 - MIT license ([LICENSE-MIT](LICENSE-MIT) or
   <http://opensource.org/licenses/MIT>)

at your option.
