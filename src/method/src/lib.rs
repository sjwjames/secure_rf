pub mod decision_tree;
pub mod random_forest;
pub trait TrainingMethod{
    fn initiate(&self);
    fn train(&self);
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
