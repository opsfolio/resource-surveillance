/*markdown
Run osquery sample against active connection
*/

select uuid, hostname from system_info;

/*markdown
Apply dynamic configuration to add supplier called "hetzner-atc"
*/

SET udi_pgp_serve_ncl_supplier = '
  hetzner-atc = {
    type = "osquery",
    mode = "local",
    atc-file-path = "./hetzner-atc.json",
    auth = [
      {
        username = "baasit",
        password = "supplier",
      },
    ],
  }';