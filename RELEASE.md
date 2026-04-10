RELEASE_TYPE: patch

Bump our pinned hegel-core to [0.4.0](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.0), incorporating the following change:

> This patch changes our CBOR tag for text fields from `6` to `91`, to avoid reserving a "Standards Action" tag, even though it is technically unassigned. See https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml.
>
> The protocol version is now `0.10`.
>
> — [v0.4.0](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.0)
