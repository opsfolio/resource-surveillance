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