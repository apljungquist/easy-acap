_A tiny library documenting common paths that ACAPs may use_

## Example

One use of the standard directories is saving a configuration file for an app:

```no_run
use std::{fs::File, io::Write};

let path = acap_dirs::localdata_dir().unwrap().join("config.json");
let mut file = File::create(&path).unwrap();
file.write_all(b"{}").unwrap();
```

For clarity the above example omits:
- Atomic writes.
- Error handling.
- Schema and serialization of the config e.g., using `serde` and `serde_json`;
