use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "proto/tendermint/types/types.proto",
            "proto/tendermint/types/canonical.proto",
            "proto/tendermint/abci/types.proto",
            "proto/cosmos/base/abci/v1beta1/abci.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
