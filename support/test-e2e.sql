/*markdown
Start UDI-PGP with a config file. Find a sample config file in support/config-full.ncl.
```bash
surveilr udi pgp -c ./support/config.ncl
```
According to the configuration file PGP will start on the port 7777. Utilizing the VSCode SQL Editor to connect to UDI-PGP, connect to the 7777 port 
with the database name being the name of the supplier, which in this case is "local-supplier".

Run osquery sample against the active connection
*/

select uuid, hostname from system_info;

/*markdown
Add a new supplier called "hetzner-atc"
*/

SET udi_pgp_serve_ncl_supplier = '
  let hetzner-atc = {
    type = "osquery",
    mode = "local",
    atc-file-path = "./hetzner-atc.json",
    auth = [
      {
        username = "baasit",
        password = "supplier",
      },
    ],
  } in hetzner-atc';

select id, person, code from party_role;

/*markdown
Add a remote supplier to the current configuration. To make this work, edit the name of the supplier(remote-supplier) and also any parameter you need to change.
*/

SET udi_pgp_serve_ncl_supplier = '
  let remote-supplier = { 
    type = "osquery",
    mode = "remote",
    ssh-targets = [
      {
        id = "one",
        host = "157.245.40.97",
        port = 22,
        user = "oshuporu"
      },
      {
        id = "me",
        host = "localhost",
        port = 222,
        user = "lilit"
      },
      {
        id = "do_test",
        host = "128.199.1.17",
        port = 22,
        user = "root"
      },
    ],
    auth = [
      {
        username = "baasit",
        password = "supplier",
      },
    ],
  } in remote-supplier';

select uuid, hostname from system_info;

/*markdown
To edit the health and port addresses for UDI-PGP, you can use the following SET variable.
*/

SET udi_pgp_serve_ncl_core = '
  let config = {  
    addr = "127.0.0.1:5555",
    metrics = 7777,
    health = 9999 
  } in config  
';