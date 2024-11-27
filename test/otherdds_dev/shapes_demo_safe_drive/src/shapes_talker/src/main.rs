use safe_drive::{context::Context, error::DynError, logger::Logger, pr_info};
use std::time::Duration;

fn main() -> Result<(), DynError> {
    // Create a context.
    let ctx = Context::new()?;

    // Create a node.
    let node = ctx.create_node("talker", None, Default::default())?;

    // Create a publisher.
    let publisher = node.create_publisher::<shapes_interfaces::msg::Shape>("Square", None)?;

    // Create a logger.
    let logger = Logger::new("shapes_talker");

    // Create a message
    let mut my_shape = create_message()?;

    loop {
        pr_info!(logger, "send: {:?}", my_shape); // Print log.

        // Send a message.
        publisher.send(&my_shape)?;

        my_shape.x += 10;
        my_shape.y += 10;
        my_shape.x = my_shape.x % 255;
        my_shape.y = my_shape.y % 255;

        std::thread::sleep(Duration::from_secs(1));
    }
}

fn create_message() -> Result<shapes_interfaces::msg::Shape, DynError> {
    let mut my_msg = shapes_interfaces::msg::Shape::new().unwrap();

    my_msg.color.assign("RED");

    my_msg.x = 0;
    my_msg.y = 0;

    my_msg.shape_size = 42;

    Ok(my_msg)
}
