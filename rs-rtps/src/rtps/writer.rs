use mio_extras::channel as mio_channel;

pub struct Writer {
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
}
pub struct WriterIngredients {
    writer_command_receiver: mio_channel::Receiver<WriterCmd>,
}
pub struct WriterCmd {}
