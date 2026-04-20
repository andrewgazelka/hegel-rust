RELEASE_TYPE: patch

This patch loosens restrictions on using threads in hegel-rust. `TestCase` now implements `Send`
(and already implemented Clone) so data generation can now occur from multiple threads.
