use lazy_static::lazy_static;

use std::collections::HashMap;
use std::str::FromStr;

use AprsError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Symbol {
    // CODE+DSTCALL+DESCRIPTION from aprs.fi APRS symbols index by Hessu, OH7LZB
    // source file: github.com/hessu/aprs-symbol-index/symbols.csv
    // license: CC BY-SA 4.0
    // changes: none
    //                            CODE DSTCALL DESCRIPTION
    PoliceStation,             // /!   BB      Police station
    NoDescriptionBC,           // /"   BC
    Digipeater,                // /#   BD      Digipeater
    Telephone,                 // /$   BE      Telephone
    DxCluster,                 // /%   BF      DX cluster
    HfGateway,                 // /&   BG      HF gateway
    SmallAircraft,             // /'   BH      Small aircraft
    MobileSatelliteStation,    // /(   BI      Mobile satellite station
    Wheelchair,                // /)   BJ      Wheelchair, handicapped
    Snowmobile,                // /*   BK      Snowmobile
    RedCross,                  // /+   BL      Red Cross
    BoyScouts,                 // /,   BM      Boy Scouts
    House,                     // /-   BN      House
    RedX,                      // /.   BO      Red X
    RedDot,                    // //   BP      Red dot
    NumberedCircle0,           // /0   P0      Numbered circle: 0
    NumberedCircle1,           // /1   P1      Numbered circle: 1
    NumberedCircle2,           // /2   P2      Numbered circle: 2
    NumberedCircle3,           // /3   P3      Numbered circle: 3
    NumberedCircle4,           // /4   P4      Numbered circle: 4
    NumberedCircle5,           // /5   P5      Numbered circle: 5
    NumberedCircle6,           // /6   P6      Numbered circle: 6
    NumberedCircle7,           // /7   P7      Numbered circle: 7
    NumberedCircle8,           // /8   P8      Numbered circle: 8
    NumberedCircle9,           // /9   P9      Numbered circle: 9
    Fire,                      // /:   MR      Fire
    Campground,                // /;   MS      Campground, tent
    Motorcycle,                // /<   MT      Motorcycle
    RailroadEngine,            // /=   MU      Railroad engine
    Car,                       // />   MV      Car
    FileServer,                // /?   MW      File server
    HurricanePredictedPath,    // /@   MX      Hurricane predicted path
    AidStation,                // /A   PA      Aid station
    Bbs,                       // /B   PB      BBS
    Canoe,                     // /C   PC      Canoe
    NoDescriptionPD,           // /D   PD
    Eyeball,                   // /E   PE      Eyeball
    FarmVehicle,               // /F   PF      Farm vehicle, tractor
    GridSquare3By3,            // /G   PG      Grid square, 3 by 3
    Hotel,                     // /H   PH      Hotel
    TcpIpNetworkStation,       // /I   PI      TCP/IP network station
    NoDescriptionPJ,           // /J   PJ
    School,                    // /K   PK      School
    PcUser,                    // /L   PL      PC user
    MacApple,                  // /M   PM      Mac apple
    NtsStation,                // /N   PN      NTS station
    Balloon,                   // /O   PO      Balloon
    PoliceCar,                 // /P   PP      Police car
    NoDescriptionPQ,           // /Q   PQ
    RecreationalVehicle,       // /R   PR      Recreational vehicle
    SpaceShuttle,              // /S   PS      Space Shuttle
    Sstv,                      // /T   PT      SSTV
    Bus,                       // /U   PU      Bus
    Atv,                       // /V   PV      ATV, Amateur Television
    WeatherServiceSite,        // /W   PW      Weather service site
    Helicopter,                // /X   PX      Helicopter
    Sailboat,                  // /Y   PY      Sailboat
    WindowsFlag,               // /Z   PZ      Windows flag
    Human,                     // /[   HS      Human
    DfTriangle,                // /\   HT      DF triangle
    Mailbox,                   // /]   HU      Mailbox, post office
    LargeAircraft,             // /^   HV      Large aircraft
    WeatherStation,            // /_   HW      Weather station
    SatelliteDishAntenna,      // /`   HX      Satellite dish antenna
    Ambulance,                 // /a   LA      Ambulance
    Bicycle,                   // /b   LB      Bicycle
    IncidentCommandPost,       // /c   LC      Incident command post
    FireStation,               // /d   LD      Fire station
    Horse,                     // /e   LE      Horse, equestrian
    FireTruck,                 // /f   LF      Fire truck
    Glider,                    // /g   LG      Glider
    Hospital,                  // /h   LH      Hospital
    Iota,                      // /i   LI      IOTA, islands on the air
    Jeep,                      // /j   LJ      Jeep
    TruckLK,                   // /k   LK      Truck
    Laptop,                    // /l   LL      Laptop
    MicERepeater,              // /m   LM      Mic-E repeater
    Node,                      // /n   LN      Node, black bulls-eye
    EmergencyOperationsCenter, // /o   LO      Emergency operations center
    Dog,                       // /p   LP      Dog
    GridSquare2By2,            // /q   LQ      Grid square, 2 by 2
    RepeaterTower,             // /r   LR      Repeater tower
    ShipOrPowerBoat,           // /s   LS      Ship, power boat
    TruckStop,                 // /t   LT      Truck stop
    SemiTrailerTruck,          // /u   LU      Semi-trailer truck, 18-wheeler
    VanLV,                     // /v   LV      Van
    WaterStation,              // /w   LW      Water station
    XUnix,                     // /x   LX      X / Unix
    HouseYagiAntenna,          // /y   LY      House, yagi antenna
    ShelterLY,                 // /z   LZ      Shelter
    NoDescriptionJ1,           // /{   J1
    NoDescriptionJ3,           // /}   J3
    Emergency,                 // \!   OB      Emergency
    NoDescriptionOC,           // \"   OC
    DigipeaterGreenStar,       // \#   OD      Digipeater, green star
    BankOrAtm,                 // \$   OE      Bank or ATM
    NoDescriptionOF,           // \%   OF
    GatewayStation,            // \&   OG      Gateway station
    CrashIncidentSite,         // \'   OH      Crash / incident site
    Cloudy,                    // \(   OI      Cloudy
    FirenetMeo,                // \)   OJ      Firenet MEO, MODIS Earth Observation
    Snow,                      // \*   OK      Snow
    Church,                    // \+   OL      Church
    GirlScouts,                // \,   OM      Girl Scouts
    HouseHfAntenna,            // \-   ON      House, HF antenna
    Ambiguous,                 // \.   OO      Ambiguous, question mark inside circle
    WaypointDestination,       // \/   OP      Waypoint destination
    Circle,                    // \0   A0      Circle, IRLP / Echolink/WIRES
    NoDescriptionA1,           // \1   A1
    NoDescriptionA2,           // \2   A2
    NoDescriptionA3,           // \3   A3
    NoDescriptionA4,           // \4   A4
    NoDescriptionA5,           // \5   A5
    NoDescriptionA6,           // \6   A6
    NoDescriptionA7,           // \7   A7
    WiFi,                      // \8   A8      802.11 WiFi or other network node
    GasStation,                // \9   A9      Gas station
    Hail,                      // \:   NR      Hail
    Park,                      // \;   NS      Park, picnic area
    Advisory,                  // \<   NT      Advisory, single red flag
    NoDescriptionNU,           // \=   NU
    RedCar,                    // \>   NV      Red car
    InfoKiosk,                 // \?   NW      Info kiosk
    Hurricane,                 // \@   NX      Hurricane, Tropical storm
    WhiteBox,                  // \A   AA      White box
    BlowingSnow,               // \B   AB      Blowing snow
    CoastGuard,                // \C   AC      Coast Guard
    DrizzlingRain,             // \D   AD      Drizzling rain
    Smoke,                     // \E   AE      Smoke, Chimney
    FreezingRain,              // \F   AF      Freezing rain
    SnowShower,                // \G   AG      Snow shower
    Haze,                      // \H   AH      Haze
    RainShower,                // \I   AI      Rain shower
    Lightning,                 // \J   AJ      Lightning
    KenwoodHt,                 // \K   AK      Kenwood HT
    Lighthouse,                // \L   AL      Lighthouse
    NoDescriptionAM,           // \M   AM
    NavigationBuoy,            // \N   AN      Navigation buoy
    Rocket,                    // \O   AO      Rocket
    Parking,                   // \P   AP      Parking
    Earthquake,                // \Q   AQ      Earthquake
    Restaurant,                // \R   AR      Restaurant
    Satellite,                 // \S   AS      Satellite
    Thunderstorm,              // \T   AT      Thunderstorm
    Sunny,                     // \U   AU      Sunny
    Vortac,                    // \V   AV      VORTAC, Navigational aid
    NwsSite,                   // \W   AW      NWS site
    Pharmacy,                  // \X   AX      Pharmacy
    NoDescriptionAY,           // \Y   AY
    NoDescriptionAZ,           // \Z   AZ
    WallCloud,                 // \[   DS      Wall Cloud
    NoDescriptionDT,           // \\   DT
    NoDescriptionDU,           // \]   DU
    Aircraft,                  // \^   DV      Aircraft
    WeatherSite,               // \_   DW      Weather site
    Rain,                      // \`   DX      Rain
    RedDiamond,                // \a   SA      Red diamond
    BlowingDust,               // \b   SB      Blowing dust, sand
    CdTriangle,                // \c   SC      CD triangle, RACES, CERTS, SATERN
    DxSpot,                    // \d   SD      DX spot
    Sleet,                     // \e   SE      Sleet
    FunnelCloud,               // \f   SF      Funnel cloud
    Gale,                      // \g   SG      Gale, two red flags
    Store,                     // \h   SH      Store
    BlackBox,                  // \i   SI      Black box, point of interest
    WorkZone,                  // \j   SJ      Work zone, excavating machine
    Suv,                       // \k   SK      SUV, ATV
    NoDescriptionSL,           // \l   SL
    ValueSign,                 // \m   SM      Value sign, 3 digit display
    RedTriangle,               // \n   SN      Red triangle
    SmallCircle,               // \o   SO      Small circle
    PartlyCloudy,              // \p   SP      Partly cloudy
    NoDescriptionSQ,           // \q   SQ
    Restrooms,                 // \r   SR      Restrooms
    ShipOrBoat,                // \s   SS      Ship, boat
    Tornado,                   // \t   ST      Tornado
    TruckSU,                   // \u   SU      Truck
    VanSV,                     // \v   SV      Van
    Flooding,                  // \w   SW      Flooding
    NoDescriptionSX,           // \x   SX
    Skywarn,                   // \y   SY      Skywarn
    ShelterSZ,                 // \z   SZ      Shelter
    Fog,                       // \{   Q1      Fog
    NoDescriptionQ3,           // \}   Q3
}

lazy_static! {
    static ref SYMBOL_MAP: HashMap<&'static str, Symbol> = vec![
        ("/!", Symbol::PoliceStation),
        ("/\"", Symbol::NoDescriptionBC),
        ("/#", Symbol::Digipeater),
        ("/$", Symbol::Telephone),
        ("/%", Symbol::DxCluster),
        ("/&", Symbol::HfGateway),
        ("/'", Symbol::SmallAircraft),
        ("/(", Symbol::MobileSatelliteStation),
        ("/)", Symbol::Wheelchair),
        ("/*", Symbol::Snowmobile),
        ("/+", Symbol::RedCross),
        ("/,", Symbol::BoyScouts),
        ("/-", Symbol::House),
        ("/.", Symbol::RedX),
        ("//", Symbol::RedDot),
        ("/0", Symbol::NumberedCircle0),
        ("/1", Symbol::NumberedCircle1),
        ("/2", Symbol::NumberedCircle2),
        ("/3", Symbol::NumberedCircle3),
        ("/4", Symbol::NumberedCircle4),
        ("/5", Symbol::NumberedCircle5),
        ("/6", Symbol::NumberedCircle6),
        ("/7", Symbol::NumberedCircle7),
        ("/8", Symbol::NumberedCircle8),
        ("/9", Symbol::NumberedCircle9),
        ("/:", Symbol::Fire),
        ("/;", Symbol::Campground),
        ("/<", Symbol::Motorcycle),
        ("/=", Symbol::RailroadEngine),
        ("/>", Symbol::Car),
        ("/?", Symbol::FileServer),
        ("/@", Symbol::HurricanePredictedPath),
        ("/A", Symbol::AidStation),
        ("/B", Symbol::Bbs),
        ("/C", Symbol::Canoe),
        ("/D", Symbol::NoDescriptionPD),
        ("/E", Symbol::Eyeball),
        ("/F", Symbol::FarmVehicle),
        ("/G", Symbol::GridSquare3By3),
        ("/H", Symbol::Hotel),
        ("/I", Symbol::TcpIpNetworkStation),
        ("/J", Symbol::NoDescriptionPJ),
        ("/K", Symbol::School),
        ("/L", Symbol::PcUser),
        ("/M", Symbol::MacApple),
        ("/N", Symbol::NtsStation),
        ("/O", Symbol::Balloon),
        ("/P", Symbol::PoliceCar),
        ("/Q", Symbol::NoDescriptionPQ),
        ("/R", Symbol::RecreationalVehicle),
        ("/S", Symbol::SpaceShuttle),
        ("/T", Symbol::Sstv),
        ("/U", Symbol::Bus),
        ("/V", Symbol::Atv),
        ("/W", Symbol::WeatherServiceSite),
        ("/X", Symbol::Helicopter),
        ("/Y", Symbol::Sailboat),
        ("/Z", Symbol::WindowsFlag),
        ("/[", Symbol::Human),
        ("/\\", Symbol::DfTriangle),
        ("/]", Symbol::Mailbox),
        ("/^", Symbol::LargeAircraft),
        ("/_", Symbol::WeatherStation),
        ("/`", Symbol::SatelliteDishAntenna),
        ("/a", Symbol::Ambulance),
        ("/b", Symbol::Bicycle),
        ("/c", Symbol::IncidentCommandPost),
        ("/d", Symbol::FireStation),
        ("/e", Symbol::Horse),
        ("/f", Symbol::FireTruck),
        ("/g", Symbol::Glider),
        ("/h", Symbol::Hospital),
        ("/i", Symbol::Iota),
        ("/j", Symbol::Jeep),
        ("/k", Symbol::TruckLK),
        ("/l", Symbol::Laptop),
        ("/m", Symbol::MicERepeater),
        ("/n", Symbol::Node),
        ("/o", Symbol::EmergencyOperationsCenter),
        ("/p", Symbol::Dog),
        ("/q", Symbol::GridSquare2By2),
        ("/r", Symbol::RepeaterTower),
        ("/s", Symbol::ShipOrPowerBoat),
        ("/t", Symbol::TruckStop),
        ("/u", Symbol::SemiTrailerTruck),
        ("/v", Symbol::VanLV),
        ("/w", Symbol::WaterStation),
        ("/x", Symbol::XUnix),
        ("/y", Symbol::HouseYagiAntenna),
        ("/z", Symbol::ShelterLY),
        ("/{", Symbol::NoDescriptionJ1),
        ("/}", Symbol::NoDescriptionJ3),
        ("\\!", Symbol::Emergency),
        ("\\\"", Symbol::NoDescriptionOC),
        ("\\#", Symbol::DigipeaterGreenStar),
        ("\\$", Symbol::BankOrAtm),
        ("\\%", Symbol::NoDescriptionOF),
        ("\\&", Symbol::GatewayStation),
        ("\\'", Symbol::CrashIncidentSite),
        ("\\(", Symbol::Cloudy),
        ("\\)", Symbol::FirenetMeo),
        ("\\*", Symbol::Snow),
        ("\\+", Symbol::Church),
        ("\\,", Symbol::GirlScouts),
        ("\\-", Symbol::HouseHfAntenna),
        ("\\.", Symbol::Ambiguous),
        ("\\/", Symbol::WaypointDestination),
        ("\\0", Symbol::Circle),
        ("\\1", Symbol::NoDescriptionA1),
        ("\\2", Symbol::NoDescriptionA2),
        ("\\3", Symbol::NoDescriptionA3),
        ("\\4", Symbol::NoDescriptionA4),
        ("\\5", Symbol::NoDescriptionA5),
        ("\\6", Symbol::NoDescriptionA6),
        ("\\7", Symbol::NoDescriptionA7),
        ("\\8", Symbol::WiFi),
        ("\\9", Symbol::GasStation),
        ("\\:", Symbol::Hail),
        ("\\;", Symbol::Park),
        ("\\<", Symbol::Advisory),
        ("\\=", Symbol::NoDescriptionNU),
        ("\\>", Symbol::RedCar),
        ("\\?", Symbol::InfoKiosk),
        ("\\@", Symbol::Hurricane),
        ("\\A", Symbol::WhiteBox),
        ("\\B", Symbol::BlowingSnow),
        ("\\C", Symbol::CoastGuard),
        ("\\D", Symbol::DrizzlingRain),
        ("\\E", Symbol::Smoke),
        ("\\F", Symbol::FreezingRain),
        ("\\G", Symbol::SnowShower),
        ("\\H", Symbol::Haze),
        ("\\I", Symbol::RainShower),
        ("\\J", Symbol::Lightning),
        ("\\K", Symbol::KenwoodHt),
        ("\\L", Symbol::Lighthouse),
        ("\\M", Symbol::NoDescriptionAM),
        ("\\N", Symbol::NavigationBuoy),
        ("\\O", Symbol::Rocket),
        ("\\P", Symbol::Parking),
        ("\\Q", Symbol::Earthquake),
        ("\\R", Symbol::Restaurant),
        ("\\S", Symbol::Satellite),
        ("\\T", Symbol::Thunderstorm),
        ("\\U", Symbol::Sunny),
        ("\\V", Symbol::Vortac),
        ("\\W", Symbol::NwsSite),
        ("\\X", Symbol::Pharmacy),
        ("\\Y", Symbol::NoDescriptionAY),
        ("\\Z", Symbol::NoDescriptionAZ),
        ("\\[", Symbol::WallCloud),
        ("\\\\", Symbol::NoDescriptionDT),
        ("\\]", Symbol::NoDescriptionDU),
        ("\\^", Symbol::Aircraft),
        ("\\_", Symbol::WeatherSite),
        ("\\`", Symbol::Rain),
        ("\\a", Symbol::RedDiamond),
        ("\\b", Symbol::BlowingDust),
        ("\\c", Symbol::CdTriangle),
        ("\\d", Symbol::DxSpot),
        ("\\e", Symbol::Sleet),
        ("\\f", Symbol::FunnelCloud),
        ("\\g", Symbol::Gale),
        ("\\h", Symbol::Store),
        ("\\i", Symbol::BlackBox),
        ("\\j", Symbol::WorkZone),
        ("\\k", Symbol::Suv),
        ("\\l", Symbol::NoDescriptionSL),
        ("\\m", Symbol::ValueSign),
        ("\\n", Symbol::RedTriangle),
        ("\\o", Symbol::SmallCircle),
        ("\\p", Symbol::PartlyCloudy),
        ("\\q", Symbol::NoDescriptionSQ),
        ("\\r", Symbol::Restrooms),
        ("\\s", Symbol::ShipOrBoat),
        ("\\t", Symbol::Tornado),
        ("\\u", Symbol::TruckSU),
        ("\\v", Symbol::VanSV),
        ("\\w", Symbol::Flooding),
        ("\\x", Symbol::NoDescriptionSX),
        ("\\y", Symbol::Skywarn),
        ("\\z", Symbol::ShelterSZ),
        ("\\{", Symbol::Fog),
        ("\\}", Symbol::NoDescriptionQ3),
    ]
    .into_iter()
    .collect();
}

impl FromStr for Symbol {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match SYMBOL_MAP.get(s) {
            Some(symbol) => Ok(symbol.to_owned()),
            None => Err(AprsError::InvalidSymbolIdentifier(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid() {
        assert_eq!("/!".parse::<Symbol>(), Ok(Symbol::PoliceStation));
    }

    #[test]
    fn parse_invalid() {
        assert_eq!(
            "'?".parse::<Symbol>(),
            Err(AprsError::InvalidSymbolIdentifier("'?".to_owned()))
        );
    }
}
