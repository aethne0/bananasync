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
        let mut runtime = Runtime::new();
        let res = runtime.block_on(async { gimme_five().await + gimme_five().await });
        assert_eq!(res, 10);
    }
}

