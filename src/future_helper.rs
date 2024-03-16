use std;

use std::future::Future;

/// Return type of wait_for_one_to_complete indicating which future completed before the other.
pub enum FirstOrSecond<A, B> {
    First(A),
    Second(B),
}

/// Wait for one of the two futures to complete and return which one completed first.
/// This is a wrapper around the select function from the futures crate for the common
/// case of returning just an output item - dropping both futures at the completion of one.
pub async fn wait_for_one_to_complete<Fut1, Fut2, Out1, Out2>(
    fut1: Fut1,
    fut2: Fut2,
) -> FirstOrSecond<Out1, Out2>
where
    Fut1: Future<Output = Out1>,
    Fut2: Future<Output = Out2>,
{
    use futures::future::{self, Either};
    match future::select(std::pin::pin!(fut1), std::pin::pin!(fut2)).await {
        Either::Left((value_1, _)) => FirstOrSecond::First(value_1),
        Either::Right((value_2, _)) => FirstOrSecond::Second(value_2),
    }
}
