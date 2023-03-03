use speedy::Readable;

#[derive(Readable)]
pub struct SubMessage {
    header: SubMessageHeader,
    body: SubMessageBody,
}

#[derive(Readable)]
struct SubMessageHeader {
    submessageId: SubMessageId,
    flags: u8,
    octetsToNextHeader: u16,
}

#[derive(Readable)]
struct SubMessageId {
    submessageId: u8,
}

#[derive(Readable)]
struct SubMessageBody {

}
