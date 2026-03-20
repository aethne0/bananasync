// todo: assert linux

mod runtime;
pub use runtime::Runtime;
pub mod time;

#[cfg(test)]
mod test {
    use crate::runtime::Runtime;

    async fn gimme_five() -> i32 {
        5
    }

    #[test]
    fn basic() {
        Runtime::block_on(async {
            let res = gimme_five().await + gimme_five().await;
            assert_eq!(res, 10);
        });
    }
}
