use anyhow::Result;
use vergen_gix::{Emitter, GixBuilder};

fn main() -> Result<()> {
    let gix = GixBuilder::default()
        .describe(true, true, None)
        .sha(true)
        .dirty(true)
        .build()?;

    Emitter::default().add_instructions(&gix)?.emit()
}
