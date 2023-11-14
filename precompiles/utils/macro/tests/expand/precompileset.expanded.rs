use {
    core::marker::PhantomData,
    precompile_utils::{EvmResult, prelude::*, testing::PrecompileTesterExt},
    sp_core::H160,
};
struct PrecompileSet<Runtime>(PhantomData<Runtime>);
type Discriminant = u32;
type GetAssetsStringLimit<R> = R;
type MockRuntime = ConstU32<42>;
