// Criterion (3): `no_traced!` refuses a field whose type is not `NoTraced`
// (stands in for a GC handle, which never implements it).
pub struct GcHandle;

token_cell::no_traced! {
    pub struct Data {
        pub n: u32,
        pub h: GcHandle,
    }
}

fn main() {}
