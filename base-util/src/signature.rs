use ring::hmac;

pub fn signature(secret: &[u8], data: &str) -> String {
    let mac = hmac::Key::new(hmac::HMAC_SHA256, secret);
    let sign = hmac::sign(&mac, data.as_bytes());
    hex::encode(sign.as_ref())
}
