# Axon Drug Injector

[![License][License Badge]][License Description]
[![GitHub Actions][GitHub Action Badge]][GitHub Actions Link]

Drug Injector for [Axon], which can connect to an [Axon] network through P2P protocols, and inject malicious messages.

> [!WARNING]
>
> This is just a simple example to show how to connect to an [Axon] network through P2P protocols.
>
> There are no plans for new features and the maintainer don't intend to respond to any issues.
>
> If you want to more features, feel free to fork it.

[License Badge]: https://img.shields.io/badge/License-MIT-blue.svg
[License Description]: https://spdx.org/licenses/MIT.html "MIT License"
[GitHub Action Badge]: https://github.com/yangby-cryptape/axon-drug-injector/workflows/CI/badge.svg
[GitHub Actions Link]: https://github.com/yangby-cryptape/axon-drug-injector/actions

## Usage

- Compile:

  ```bash
  cargo build --release
  ```

- Create a configuration file.

  There is a template of the configuration file [`config-template.toml`].

  You can update it to adapt the [Axon] network you want to connect.

- Run the service.

  ```bash
  ./target/release/axon-drug-injector serve -c config.toml
  ```

  Tips:

  - Environment variable `RUST_LOG` can be used to control logging.

    Ref: [Crate `env_logger` / Enabling logging](https://docs.rs/env_logger/latest/env_logger/#enabling-logging)

- Use a normal [Axon] JSON-RPC APIs provider to build a signed transaction.

  You can ignore all server-side restrictions which may be restricted by normal Axon nodes, for example, [`max_gas_cap`].

- Send the signed transaction to your Axon Drug Injector service through the JSON-RPC method `eth_sendRawTransaction`.

  Then, no matter whether this transaction is valid or invalid, it will be broadcasted to the connected Axon nodes through P2P protocols.

  And, all server-side restrictions will be bypassed.

- Enjoy it!

[`config-template.toml`]: etc/config-template.toml
[`max_gas_cap`]: https://github.com/axonweb3/axon/blob/6a574cdbe0b0f826968602d253721606f2cd5ded/devtools/chain/config.toml#L18

## Examples

- [An example, written in TypeScript](examples/transfer.ts),
  to inject the simplest transfer transaction into an Axon network.

## License

Licensed under [MIT License][MIT].

[MIT]: LICENSE

[Axon]: https://axonweb3.io/
