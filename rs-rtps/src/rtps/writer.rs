use mio_channel;

pub struct Writer {
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
}
pub struct WriterIngredients {
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
}
pub struct WriterCmd {}
