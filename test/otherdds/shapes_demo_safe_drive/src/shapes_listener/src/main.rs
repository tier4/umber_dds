use safe_drive::{context::Context, error::DynError, logger::Logger, pr_info};
use shapes_interfaces;

fn main() -> Result<(), DynError> {
    // Create a context.
    let ctx = Context::new()?;

    // Create a node.
    let node = ctx.create_node("listener", None, Default::default())?;

    // Create a subscriber.
    let subscriber = node.create_subscriber::<shapes_interfaces::msg::Shape>("Square", None)?;

    // Create a logger.
    let logger = Logger::new("shape_listener");

    // Create a selector.
    let mut selector = ctx.create_selector()?;

    pr_info!(logger, "listening");

    // Add a callback function.
    selector.add_subscriber(
        subscriber,
        Box::new(move |msg| {
            pr_info!(
                logger,
                "Shape {{ color: {}, x: {}, y: {}, size: {} }}",
                msg.color,
                msg.x,
                msg.y,
                msg.shape_size
            );
        }),
    );

    // Spin.
    loop {
        selector.wait()?;
    }
}
