
aprs-parser
==============================================================================

[![Build Status](https://travis-ci.org/Turbo87/aprs-parser-rs.svg?branch=master)](https://travis-ci.org/Turbo87/aprs-parser-rs)

[APRS] message parser for [Rust]

[APRS]: http://www.aprs.org/
[Rust]: https://www.rust-lang.org/

Features
--------------------------------------
- Supports packet encoding and decoding
- Supports textual representations (APRS-IS) as well as binary/AX.25 representations (KISS)

Usage
------------------------------------------------------------------------------

```rust
extern crate aprs_parser;

fn main() {
    let result = aprs_parser::parse(
        br"ICA3D2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054"
    );

    println!("{:#?}", result);

    // Ok(
    //     AprsPacket {
    //         from: Callsign {
    //             call: "IC17F2"
    //             ssid: None,
    //         },
    //         to: Callsign {
    //             call: [
    //                 65,
    //                 80,
    //                 82,
    //                 83,
    //             ],
    //             ssid: None,
    //         },
    //         via: [
    //             QConstruct(
    //                 AS,
    //             ),
    //             Callsign(
    //                 Callsign {
    //                     call: "dl4mea"
    //                     ssid: None,
    //                 },
    //                 false,
    //             ),
    //         ],
    //         data: Position(
    //             AprsPosition {
    //                 timestamp: Some(
    //                     HHMMSS(
    //                         7,
    //                         48,
    //                         49,
    //                     ),
    //                 ),
    //                 messaging_supported: false,
    //                 latitude: Latitude(
    //                     48.36016666666667,
    //                 ),
    //                 longitude: Longitude(
    //                     12.408166666666666,
    //                 ),
    //                 precision: HundredthMinute,
    //                 symbol_table: '\\',
    //                 symbol_code: '^',
    //                 comment: [
    //                     51,
    //                     50,
    //                     50,
    //                     47,
    //                     49,
    //                     48,
    //                     51,
    //                     47,
    //                     65,
    //                     61,
    //                     48,
    //                     48,
    //                     51,
    //                     48,
    //                     53,
    //                     52,
    //                 ],
    //                 cst: Uncompressed,
    //             },
    //         ),
    //     },
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
