/*markdown
Start UDI-PGP with a config file. Find a sample config file in support/config-full.ncl.

```bash
surveilr udi pgp -c ./support/config.ncl
```
According to the configuration file PGP will start on the port 7777. Utilizing the VSCode SQL Editor to connect to UDI-PGP, connect to the 7777 port. Since the intitial configuration is devoid of any supplier, we don't need a database name yet as is a username and and passowrd, they can all be left empty.

Run osquery sample against the active connection to get the list of suppliers which should be empty.
*/

SELECT * FROM udi_pgp_supplier;

/*markdown
Query UDI-PGP for all other configuration parameters besides the suppliers, like the port and address PGP is bound to, the health and metrics addresses.
*/

SEELCT * FROM udi_pgp_config;

/*markdown
To add a supplier to UDI-PGP, we can utilize `SET` queries with specific variable names. For example, to add a new local supplier to the existing configuration, execute the below cell
*/

SET udi_pgp_serve_ncl_supplier = '
  let local-supplier = {
    type = "osquery",
    mode = "local",
    atc-file-path = "./hetzner-atc.json",
    auth = [
      {
        username = "john",
        password = "pass",
      },
    ],
  } in local-supplier';

/*markdown
Initiate a new connection to UDI-PGP through the SQL Notebook editor, set the database name to "local-supplier" and add the password as "pass" and username as "john" (the authentication details are described in the auth section of a supplier). Then, execute the below query against the new connection to UDI-PGP.
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
        username = "john",
        password = "doe",
      },
    ],
  } in hetzner-atc';

/*markdown
Before executing the query, initiate another session with UDI-PGP through the SQL Notebook side panel. Use "hetzner" as the name of the database and fill in the username and password sections as described in the auth filed.
*/

select id, person, code from party_role;

/*markdown
Adding a remote supplier to the current configuration. To make this work, edit the name of the supplier(remote-supplier) and also any parameter you need to change.
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
        username = "john",
        password = "doe",
      },
    ],
  } in remote-supplier';

/*markdown
Before executing the query, initiate another session with UDI-PGP through the SQL Notebook side panel. Use "remote-supplier" as the name of the database and fill in the username and password sections as described in the auth filed.
*/

select uuid, hostname from system_info;

/*markdown
Core UDI-PGP configuration like the health address and the adress of the metrics port can be updated. The "addr" parameter is always ignored. 
*/

SET udi_pgp_serve_ncl_core = '
  let config = {  
    addr = "127.0.0.1:5555",
    metrics = "127.0.0.1:3333",
    health = "127.0.0.1:5555" 
  } in config  
';