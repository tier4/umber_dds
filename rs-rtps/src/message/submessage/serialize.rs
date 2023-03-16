
// TODO: 各ヴァリアントに構造体とflagを持たせる
pub enum EntitySubmessage {
    Data,
    DataFrag,
    HeartBeat,
    Gap,
    AckNack,
    NackFrag,
}

pub enum InterpreterSubmessage{
    InfoSource,
    InfoDestinatio,
    InfoTImestamp,
    InfoReply,
    Pad,
}
