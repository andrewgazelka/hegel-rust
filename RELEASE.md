RELEASE_TYPE: patch

Bump our pinned hegel-core to [0.3.0](https://github.com/hegeldev/hegel-core/releases/tag/v0.3.0), incorporating the following changes:

> Add protocol support for reporting failure blobs back to the client. These are strings that can be used to reproduce a specific failure exactly.
>
> — [v0.2.4](https://github.com/hegeldev/hegel-core/releases/tag/v0.2.4)

> This patch changes how `const`, `sampled_from`, and `one_of` are defined in the protocol, to harmonize with the other generator definitions:
>
> - `{"const": value}` is now `{"type": "constant", "value": value}`
> - `{"sampled_from": [...]}` is now `{"type": "sampled_from", "values": [...]}`
> - `{"one_of": [...]}` is now `{"type": "one_of", "generators": [...]}`
>
> As a result, this patch bumps our protocol version to `0.8`.
>
> — [v0.2.5](https://github.com/hegeldev/hegel-core/releases/tag/v0.2.5)

> Several breaking changes:
> - Rename `channel` to `stream` everywhere.
> - Restructure parameters and return values for `collection` commands.
>
> — [v0.3.0](https://github.com/hegeldev/hegel-core/releases/tag/v0.3.0)
