pub(crate) trait StatusCode {
    fn pretty_print(&self) -> String;

    fn get_all_100() -> Vec<u16>;

    fn get_all_200() -> Vec<u16>;

    fn get_all_300() -> Vec<u16>;

    fn get_all_400() -> Vec<u16>;

    fn get_all_500() -> Vec<u16>;

    fn get_all_status_codes() -> Vec<u16>;
}

impl StatusCode for u16 {
    fn get_all_100() -> Vec<u16> {
        let mut res = Vec::new();
        res.extend_from_slice(&[100u16, 101u16, 102u16, 103u16]);
        res
    }

    fn get_all_200() -> Vec<u16> {
        let mut res = Vec::new();
        res.extend_from_slice(&[
            200u16, 201u16, 202u16, 203u16, 204u16, 205u16, 206u16, 207u16, 208u16, 226u16,
        ]);
        res
    }

    fn get_all_300() -> Vec<u16> {
        let mut res = Vec::new();
        res.extend_from_slice(&[300u16, 301u16, 302u16, 303u16, 304u16, 307u16, 308u16]);
        res
    }

    fn get_all_400() -> Vec<u16> {
        let mut res = Vec::new();
        res.extend_from_slice(&[
            400u16, 401u16, 402u16, 403u16, 404u16, 405u16, 406u16, 407u16, 408u16, 409u16, 410u16,
            411u16, 412u16, 413u16, 414u16, 415u16, 416u16, 417u16, 418u16, 421u16, 422u16, 423u16,
            424u16, 425u16, 426u16, 428u16, 429u16, 431u16, 451u16,
        ]);
        res
    }

    fn get_all_500() -> Vec<u16> {
        let mut res = Vec::new();
        res.extend_from_slice(&[
            500u16, 501u16, 502u16, 503u16, 504u16, 505u16, 506u16, 507u16, 508u16, 510u16, 511u16,
        ]);
        res
    }

    fn get_all_status_codes() -> Vec<u16> {
        let http_100 = Self::get_all_100();
        let http_200 = Self::get_all_200();
        let http_300 = Self::get_all_300();
        let http_400 = Self::get_all_400();
        let http_500 = Self::get_all_500();

        let mut res = Vec::new();

        res.extend(http_100);
        res.extend(http_200);
        res.extend(http_300);
        res.extend(http_400);
        res.extend(http_500);

        res
    }

    fn pretty_print(&self) -> String {
        match self {
            100u16..=199u16 => format!("⚪️ {self}"),
            200u16..=299u16 => format!("🟢 {self}"),
            300u16..=399u16 => format!("🔵 {self}"),
            400u16..=499u16 => format!("🟠 {self}"),
            500u16..=599u16 => format!("🔴 {self}"),
            _ => "Unknown".to_string(),
        }
    }
}
