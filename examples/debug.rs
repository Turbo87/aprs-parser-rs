use aprs_parser::{AprsData, AprsPacket};

extern crate aprs_parser;

fn main() {

    let packets = vec![
        r##"WA6IFI-12>APN391,PVLY,WIDE2-1:!3846.33N110459.55W#PHG3830 WA6IFI W2,COn /A12349"##,
        r##"KC5W>SYSRSQ,W0NED,WIDE1,WIDE2-1:`qXxl -/`_4"##,
        r##"K0MKX>SYTPUV,WD4IXD-2,WIDE1,N0SZ-2,WIDE2:`q`Ul Q./`"G@}Say hello 146.52...._4"##,
        r##"AE0SS-2>APZEOS,EOSS,WIDE2-1:/175008h3902.30N/10308.43WO287/000/A=005143 320T8472P EOSS BALLOON"##,
        r##"WD0AJG-6>SY0SSW,WA6IFI-12,PVLY,WIDE2:`pO$r]Sj/`"Jv}OnTheRoadAgain_%"##,
        r##"N0SZ-2>APMI06,W0NED,NCFPD,WIDE2:@211337z3915.89NI10506.85W#PHG5130/Devils Head Digi/I-Gate12.3V,26.3C/79.3F/A=009150"##,
        r##"IC17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054"##,
        r##"N0SZ-2>APMI06,W0NED,NCFPD,WIDE2:@242017z3915.89NI10506.85W#PHG5130/Devils Head Digi/I-Gate12.3V,27.1C/80.7F/A=009150"##,
        r##"WA0DE-9>S9RSVQ,WIDE1-1,WIDE2-1:`pDMx#/"J%}ARESDEC CommTrailer"##,
        r##"K0RAP-9>SYTWZZ,W0NED,WIDE1:`pKo>(>/`"F(}147.210MHz C100 +060 http://www.k0rap.com_4"##,
        r##"WD0AJG-9>SY3SQR,WQ8M-9,WIDE1,WIDE2-1:`pQPmp>/`"GO}OnTheRoadAgain-146.52 or scan local RPT_1"##,
        r##"W8XAL-10>TPSTWW,W0NED,WIDE1,NCFPD,WIDE2:`pWvl!:>\`"3O}438.025MHz C088 -500 The truth is out there. _1"##,
    ];

    for p in packets {
        println!("\n=========================");
        println!("PACKET:  {:?}", p);

        /*
        match String::from_utf8(p.into()) {
            Ok(s) => println!("utf8: {}", s),
            Err(e) => println!("Unable to convert to utf-8: {}", e),
        };
        */

        if let Ok(packet) = AprsPacket::decode_textual(p.as_bytes()) {
            println!("{:?}\n", &packet);

            match packet.data {
                AprsData::Position(pos_packet) => {
                    if let Some(alt) = pos_packet.position.altitude {
                        println!("ALTITUDE: {}", alt.altitude_feet());
                    }
                },
                AprsData::MicE(mice_packet) => {
                    if let Some(alt) = mice_packet.altitude {
                        println!("ALTITUDE: {}", alt.altitude_feet());
                    }
                },
                _ => println!("Unknown packet"),
            }
        };
    }
}
