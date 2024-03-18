/*markdown
Start UDI-PGP with a config file. Find a sample config file in support/config-full.ncl.
`surveilr udi pgp -c ./support/config-full.ncl`
According to the configuration file PGP will start on the port 7777. Utilizing the VSCode SQL Editor to connect to UDI-PGP, connect to the 7777 port. Since the intitial configuration is devoid of any supplier, we don't need a database name yet as is a username and and passowrd, they can all be left empty.
Run osquery sample against the active connection to get the list of suppliers which should be empty.
*/

SELECT * FROM udi_pgp_supplier; -- Show current suppliers, at start, it should be empty

/*markdown
Query UDI-PGP for all other configuration parameters besides the suppliers, like the port and address PGP is bound to, the health and metrics addresses.
*/

SELECT * FROM udi_pgp_config; -- Show the configuration for UDI-PGP, that is, the address it is bound to and the metrics and health addresses.

SELECT * FROM udi_pgp_observe_query_exec; -- Show log entries, it should show result of the two queries ran earlier

/*markdown
To add a supplier to UDI-PGP, we can utilize `SET` queries with specific variable names. For example, to add a new supplier called `local-supplier` (a supplier can be named anything) to the existing configuration, execute the below cell
*/

SET udi_pgp_serve_ncl_supplier = '
  let local-supplier = {
    type = "osquery",
    mode = "local",
    auth = [
      {
        username = "john",
        password = "pass",
      },
    ],
  } in local-supplier';

/*markdown
NOTE: After adding a supplier, executing a query against the base configuration connection(the one created at the start of the file) will result in an error since it is now invalid. It is invalid due to the reason that it has no authentication mechanism and UDI-PGP only allows passwordless connections when there are no suppliers. 
*/

SELECT * FROM udi_pgp_supplier;

/*markdown
Initiate a new connection to UDI-PGP through the SQL Notebook editor, set the database name to "local-supplier" and add the password as "pass" and username as "john" (the authentication details are described in the auth section of a supplier). Then, execute the below query against the new connection to UDI-PGP which should return a row with a `uuid` and `hostname` columns.
*/

select uuid, hostname from system_info;

/*markdown
Add a new supplier called `hetzner` and introduce a known error by not supplying the `type`. You should see an error message stating that it failed to parse supplier due to a missing field.
*/

SET udi_pgp_serve_ncl_supplier = '
  let hetzner = {
    mode = "local",
    auth = [
      {
        username = "john",
        password = "doe",
      },
    ],
  } in hetzner';

SELECT * FROM udi_pgp_supplier; -- You should still have one supplier namely `local-supplier`, `hetzner-atc` should not be created because it's invalid

SELECT * FROM udi_pgp_observe_query_exec; -- Show log entries

/*markdown
Add a new supplier called "hetzner-atc" with an invalid file path to the ATC file. This statement should fail due to the invalid ATC file.
*/

SET udi_pgp_serve_ncl_supplier = '
  let hetzner-atc = {
    type = "osquery",
    mode = "local",
    atc-file-path = "../atc/opsfolio.sqla.-atc.json",
    auth = [
      {
        username = "john",
        password = "doe",
      },
    ],
  } in hetzner-atc';

SELECT * FROM udi_pgp_supplier; -- You should see only one supplier because the above query failed

SELECT * FROM udi_pgp_observe_query_exec; -- Show log entries to see the errors that happened during creation of the supplier

/*markdown
Before executing the query, initiate another session with UDI-PGP through the SQL Notebook side panel. Use "hetzner" as the name of the database and fill in the username and password sections as described in the auth filed. After you execute this, a schema definition error will be returned due to the incorrect ATC file.
*/

select id, person, code from party_roles;

/*markdown
Adding a remote supplier to the current configuration. To make this work, edit the name of the supplier(remote-supplier) and also any parameter you need to change. Ommiting a field like the `id` or `host` should display the appropriate error with the only exception being the port. If no port is passed, the default 22 port is assumed.
*/

SET udi_pgp_serve_ncl_supplier = '
  let remote-supplier = { 
    type = "osquery",
    mode = "remote",
    ssh-targets = [
      {
        id = "one",
        host = "157.",
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

SELECT * FROM udi_pgp_supplier;

/*markdown
Before executing the query, initiate another session with UDI-PGP through the SQL Notebook side panel. Use "remote-supplier" as the name of the database and fill in the username and password sections as described in the auth filed.
*/

select uuid, hostname from system_info;

/*markdown
Show log entries, the `exec_msg` field should be populated because the two of the specified SSH targets are invalid
*/

SELECT * FROM udi_pgp_observe_query_exec;

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

SELECT * FROM udi_pgp_config; -- Show the configuration for UDI-PGP, that is, the address it is bound to and the metrics and health addresses.