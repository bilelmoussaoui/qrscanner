#[derive(Debug)]
pub struct QRCode {
    pub width: i32,
    pub height: i32,
    pub items: Vec<Vec<bool>>,
}

impl QRCode {
    pub fn from_string(string: &str) -> Self {
        let code = qrcode::QrCode::new(string.as_bytes()).unwrap();
        let items = code
            .render::<char>()
            .quiet_zone(false)
            .module_dimensions(1, 1)
            .build()
            .split('\n')
            .into_iter()
            .map(|line| {
                line.chars()
                    .into_iter()
                    .map(|c| !c.is_whitespace())
                    .collect::<Vec<bool>>()
            })
            .collect::<Vec<Vec<bool>>>();

        let width = items.get(0).unwrap().len() as i32;
        let height = items.len() as i32;
        Self {
            width,
            height,
            items,
        }
    }
}
