# raft-stateright

```shell
cargo run --release --package raft-stateright --bin raft-sr -- help 
````

```text
Usage: raft-sr <COMMAND>

Commands:
  explore  
  check    
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```text
Usage: explore [OPTIONS] --server-count <SERVER_COUNT> --network <NETWORK> --max-term <MAX_TERM>

Options:
      --server-count <SERVER_COUNT>  
      --network <NETWORK>            [possible values: ordered, unordered-duplicating, unordered-non-duplicating]
      --lossy-network                
      --max-term <MAX_TERM>          
  -h, --help                         Print help
  -V, --version                      Print version
```

Example:

```shell
cargo run --release --package raft-stateright --bin raft-sr -- explore --server-count 2 --network unordered-duplicating --lossy-network --max-term 2 
```