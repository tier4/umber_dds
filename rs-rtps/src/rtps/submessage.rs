use speedy::Readable;

#[derive(Readable)]
pub struct SubMessage {
    header: SubMessageHeader,
    body: SubMessageBody,
}

#[derive(Readable, Debug)]
pub struct SubMessageHeader {
    submessageId: SubMessageId,
    flags: u8,
    pub octetsToNextHeader: u16,
}

#[derive(Readable, Debug)]
struct SubMessageId {
    submessageId: u8,
}

#[derive(Readable)]
struct SubMessageBody {

}
