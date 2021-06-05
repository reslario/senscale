use derive_try_from_primitive::TryFromPrimitive;

#[derive(Debug, TryFromPrimitive)]
#[repr(usize)]
pub enum Message {
    Stop = 0x1F6D1,
    Reload = 0x2672
}
